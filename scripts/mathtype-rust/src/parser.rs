use crate::ast::*;
use crate::generated::char_tables::{
    BIG_SYMBOL_COMMAND_CHARS, COMMAND_SPECIFIC_CHARS, DELIMITER_COMMAND_CHARS,
    SUM_OPERATOR_COMMAND_CHARS, TEX_COMMAND_CHARS, TEX_COMMAND_SEQUENCES, TEX_COMMAND_TEXTS,
};
use crate::generated::raw_text_tables::{literal_raw_text_override, LiteralOverrideFragment};
use crate::mathtype_ansi::encode_mathtype_text;
use crate::raw_fallback::should_force_raw_simple_command;
use std::collections::HashMap;

#[path = "parser/commands.rs"]
mod commands;
#[path = "parser/environments.rs"]
mod environments;
#[path = "parser/helpers.rs"]
mod helpers;
#[path = "parser/macros.rs"]
mod macros;
#[path = "parser/state.rs"]
mod state;
#[path = "parser/text_mode.rs"]
mod text_mode;

use helpers::*;

const MATHTYPE_TEXT_TRANSLATION_FAILED: &str = "(Tex translation failed)";

/// Strip math delimiters and keep multiline source records stable for MathType.
pub(crate) fn normalize_latex(input: &str) -> String {
    let text = input.trim().trim_start_matches('\u{feff}').trim();
    let (body, stripped_delimiters) =
        if text.starts_with("$$") && text.ends_with("$$") && text.len() >= 4 {
            (text[2..text.len() - 2].trim(), true)
        } else if text.starts_with('$') && text.ends_with('$') && text.len() >= 2 {
            (text[1..text.len() - 1].trim(), true)
        } else {
            (text, false)
        };
    if stripped_delimiters {
        // Existing MathType references were regenerated from stripped snippets with CRLF.
        body.replace("\r\n", "\n")
            .replace('\r', "\n")
            .replace('\n', "\r\n")
    } else {
        // File-based helper input without outer math delimiters preserves source line endings.
        body.to_string()
    }
}

pub(crate) struct Parser {
    chars: Vec<char>,
    pos: usize,
    macros: HashMap<String, MacroDefinition>,
    expansion_depth: usize,
    pending_raw_ws: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MacroDefinition {
    params: usize,
    replacement: String,
    render_mode: MacroRenderMode,
}

#[derive(Default)]
struct ParsedEnvironmentRows {
    rows: Vec<Vec<Expr>>,
    row_leading: Vec<String>,
    separator_leading: Vec<Vec<String>>,
    end_leading: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LimitModifier {
    Limits,
    NoLimits,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MacroRenderMode {
    RawOnly,
}

#[derive(Clone, Debug)]
enum InfixCommand {
    Over,
    Above(Expr),
    Atop,
    Choose,
    Brace,
    Brack,
}

#[derive(Clone, Debug)]
enum LeftRightDelimiter {
    Char(char),
    RawCommand(String),
}

impl Parser {
    /// Create a parser for the currently supported TeX math subset.
    pub(crate) fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            macros: HashMap::new(),
            expansion_depth: 0,
            pending_raw_ws: false,
        }
    }

    /// Parse the full formula and reject trailing unsupported syntax.
    pub(crate) fn parse(mut self) -> Result<Expr, String> {
        let expr = self.parse_sequence(None)?;
        self.skip_ws();
        if self.pos != self.chars.len() {
            return Err(format!("unexpected character {:?}", self.peek()));
        }
        Ok(expr)
    }

    /// Parse a sequence until an optional closing delimiter is reached.
    fn parse_sequence(&mut self, until: Option<char>) -> Result<Expr, String> {
        let mut items = Vec::new();
        loop {
            let had_leading_ws = self.consume_ws();
            if self.pos >= self.chars.len() || until.is_some_and(|end| self.peek() == Some(end)) {
                break;
            }
            if let Some(infix) = self.consume_infix_command() {
                let left = Expr::Sequence(items);
                let right = self.parse_sequence(until)?;
                return Ok(infix_expr(infix, left, right));
            }
            let atom = with_leading_raw_space(self.parse_complete_atom()?, had_leading_ws);
            if expr_is_empty_sequence(&atom) {
                continue;
            }
            items.push(atom);
        }
        if let Some(end) = until {
            self.expect(end)?;
        }
        Ok(Expr::Sequence(items))
    }

    /// Parse one atom, including a small set of control words.
    fn parse_atom(&mut self) -> Result<Expr, String> {
        let had_leading_ws = self.consume_ws() || self.take_pending_raw_ws();
        match self.peek() {
            Some('{') => {
                self.pos += 1;
                self.parse_sequence(Some('}'))
            }
            Some('\\') => self.parse_command(had_leading_ws),
            Some(ch) if ch != '}' => {
                self.pos += 1;
                parse_literal_char(ch, had_leading_ws)
            }
            other => Err(format!("expected atom, found {other:?}")),
        }
    }

    /// Parse subscript/superscript arguments, accepting either groups or atoms.
    fn parse_script_arg(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if self.peek() == Some('{') {
            self.pos += 1;
            Ok(collapse_single_sequence(self.parse_sequence(Some('}'))?))
        } else {
            self.parse_atom()
        }
    }

    /// Parse supported LaTeX commands that map directly to MTEF templates.
    fn parse_command(&mut self, had_leading_ws: bool) -> Result<Expr, String> {
        self.expect('\\')?;
        let start = self.pos;
        while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
            self.pos += 1;
        }
        let command: String = self.chars[start..self.pos].iter().collect();
        if command != "def" && command != "gdef" {
            if let Some(expanded) = self.expand_macro_command(&command)? {
                return Ok(with_leading_raw_space(expanded, had_leading_ws));
            }
        }
        if command == "big" {
            self.skip_ws();
            if self.peek().is_some() {
                // MathType consumes `\big` and prefixes the following visible delimiter with a
                // line marker instead of preserving the command name itself.
                return Ok(Expr::MarkedChar(
                    self.parse_delimiter_char("big delimiter")?,
                ));
            }
        }
        if ignored_delimiter_size_command(&command) {
            // MathType ignores these standalone delimiter-size hints and keeps only the visible
            // delimiter token that follows, so do not preserve the command itself in the AST.
            return Ok(Expr::Sequence(Vec::new()));
        }
        // MathType keeps some zero-argument aliases as raw source text even when scripts or
        // neighboring atoms follow, so preserve the command token before normal parsing.
        if should_force_raw_simple_command(&command) {
            return Ok(with_leading_raw_space(
                Expr::RawTex(format!("\\{command}")),
                had_leading_ws,
            ));
        }
        let expr = match command.as_str() {
            "frac" => {
                let numerator = self.parse_required_group_or_atom("fraction numerator")?;
                let denominator = self.parse_required_group_or_atom("fraction denominator")?;
                Ok(Expr::Fraction(Box::new(numerator), Box::new(denominator)))
            }
            "dfrac" => {
                let numerator = self.parse_required_group_or_atom("fraction numerator")?;
                let denominator = self.parse_required_group_or_atom("fraction denominator")?;
                let content = Expr::Fraction(Box::new(numerator), Box::new(denominator));
                Ok(Expr::Style {
                    kind: StyleKind::Display,
                    content: Box::new(content),
                })
            }
            "tfrac" => {
                let numerator = self.parse_required_group_or_atom("fraction numerator")?;
                let denominator = self.parse_required_group_or_atom("fraction denominator")?;
                let content = Expr::Fraction(Box::new(numerator), Box::new(denominator));
                Ok(Expr::Style {
                    kind: StyleKind::Text,
                    content: Box::new(content),
                })
            }
            "cfrac" => {
                let numerator =
                    self.parse_required_group_or_atom("continued fraction numerator")?;
                let denominator =
                    self.parse_required_group_or_atom("continued fraction denominator")?;
                Ok(Expr::Fraction(Box::new(numerator), Box::new(denominator)))
            }
            "sqrt" => {
                let index = self.parse_optional_bracket_group()?;
                let radicand = self.parse_required_group("square-root radicand")?;
                Ok(if let Some(index) = index {
                    Expr::NthRoot {
                        index: Box::new(index),
                        radicand: Box::new(radicand),
                    }
                } else {
                    Expr::Sqrt(Box::new(radicand))
                })
            }
            "boxed" => self.parse_raw_prefix_group_command("boxed content", command.as_str()),
            "sum" => Ok(Expr::BigOp {
                kind: BigOpKind::Sum,
                lower: None,
                upper: None,
                body: None,
            }),
            "prod" => Ok(Expr::BigOp {
                kind: BigOpKind::Product,
                lower: None,
                upper: None,
                body: None,
            }),
            "coprod" => Ok(Expr::BigOp {
                kind: BigOpKind::Coproduct,
                lower: None,
                upper: None,
                body: None,
            }),
            "bigcup" => Ok(Expr::BigOp {
                kind: BigOpKind::Union,
                lower: None,
                upper: None,
                body: None,
            }),
            "bigcap" => Ok(Expr::BigOp {
                kind: BigOpKind::Intersection,
                lower: None,
                upper: None,
                body: None,
            }),
            "int" | "intop" => Ok(Expr::Integral {
                kind: IntegralKind::Single,
            }),
            "iint" => Ok(Expr::Integral {
                kind: IntegralKind::Double,
            }),
            "iiint" => Ok(Expr::Integral {
                kind: IntegralKind::Triple,
            }),
            "oint" => Ok(Expr::Integral {
                kind: IntegralKind::Contour,
            }),
            "oiint" | "oiiint" => Ok(Expr::RawTex(format!("\\{command}"))),
            "binom" => {
                let upper = self.parse_required_group("binomial upper")?;
                let lower = self.parse_required_group("binomial lower")?;
                Ok(Expr::Pile {
                    kind: PileKind::Binom,
                    upper: Box::new(upper),
                    lower: Box::new(lower),
                })
            }
            "dbinom" => self.parse_styled_binom(StyleKind::Display),
            "tbinom" => self.parse_styled_binom(StyleKind::Text),
            "genfrac" => self.parse_genfrac(),
            "substack" => Ok(Expr::Substack {
                rows: self.parse_row_stack_group("substack content")?,
            }),
            "begin" => {
                let name = self.parse_raw_group("environment name")?;
                self.parse_environment(&name)
            }
            "left" => {
                let left = self.parse_left_right_delimiter("left delimiter")?;
                let content = self.parse_sequence_until_right()?;
                let right = self.parse_right_delimiter_spec()?;
                Ok(match (left, right) {
                    (LeftRightDelimiter::Char(left), LeftRightDelimiter::Char(right)) => {
                        Expr::Delimited {
                            left,
                            right,
                            content: Box::new(content),
                        }
                    }
                    (left, right) => {
                        let mut items = Vec::new();
                        push_left_right_delimiter(&mut items, "left", left);
                        push_visible_items(&mut items, content);
                        push_left_right_delimiter(&mut items, "right", right);
                        Expr::Sequence(items)
                    }
                })
            }
            "middle" => self.parse_middle_delimiter(),
            "!" => Ok(Expr::Space(0x01)),
            "," => Ok(Expr::Space(0x08)),
            "text" => Ok(text_mode::parse_content(
                &self.parse_raw_group("text content")?,
            )),
            "htmlId" | "htmlClass" | "htmlStyle" | "htmlData" => {
                self.parse_html_wrapper_content(command.as_str())
            }
            "textcolor" => {
                let color_name = self.parse_raw_group("text color name")?;
                let content = self.parse_visible_wrapper_group("textcolor content")?;
                let mut args = Vec::with_capacity(2);
                let raw_prefix = if let Some(rest) = color_name.strip_prefix('#') {
                    if !rest.is_empty() {
                        args.push(self.parse_visible_wrapper_text(rest)?);
                    }
                    "\\textcolor#".to_string()
                } else {
                    args.push(self.parse_visible_wrapper_text(&color_name)?);
                    "\\textcolor".to_string()
                };
                args.push(content);
                Ok(raw_prefix_sequence_with_raw(raw_prefix, args))
            }
            "colorbox" => self.parse_color_box_content(false),
            "fcolorbox" => self.parse_color_box_content(true),
            "color" => {
                let name = self.parse_raw_group("color name")?;
                Ok(Expr::Color {
                    name,
                    content: Box::new(self.parse_switch_content("colored content")?),
                })
            }
            "operatorname" => {
                let starred = self.consume_optional_star();
                if starred {
                    let name = self.parse_visible_wrapper_group("operator name")?;
                    Ok(Expr::Sequence(vec![
                        Expr::FunctionName("*".to_string()),
                        name,
                    ]))
                } else {
                    let raw = self.parse_raw_group("operator name")?;
                    Ok(Expr::FunctionName(raw))
                }
            }
            "operatornamewithlimits" => Ok(raw_prefix_expr(
                "operatornamewithlimits",
                self.parse_visible_wrapper_group("operator name")?,
            )),
            "displaystyle" | "textstyle" | "scriptstyle" | "scriptscriptstyle" => Ok(Expr::Style {
                kind: style_command_kind(command.as_str()),
                content: Box::new(self.parse_switch_content("style switch content")?),
            }),
            "char" => self.parse_char_code(),
            "verb" => self.parse_verb_literal(),
            "def" | "gdef" => self.parse_macro_definition(&command),
            "let" | "futurelet" => Ok(Expr::RawTex(self.consume_raw_primitive(command))),
            "arg" | "argmax" | "argmin" | "arccos" | "arcctg" | "arcsin" | "arctan" | "arctg"
            | "ch" | "cos" | "cosec" | "cosh" | "cot" | "cotg" | "coth" | "csc" | "ctg" | "cth"
            | "deg" | "det" | "dim" | "exp" | "gcd" | "hom" | "injlim" | "ker" | "lg" | "lim"
            | "inf" | "liminf" | "limsup" | "ln" | "log" | "max" | "min" | "plim" | "projlim"
            | "sec" | "sh" | "sin" | "sinh" | "sup" | "tan" | "tanh" | "tg" | "th" | "Pr" => {
                Ok(Expr::FunctionName(command))
            }
            // These four commands are layout variants around the visible "lim" token,
            // not plain function names, so keep the structural accent/bar information.
            "varinjlim" => Ok(Expr::ArrowAccent {
                kind: ArrowAccentKind::Right,
                under: true,
                content: Box::new(Expr::FunctionName("lim".to_string())),
            }),
            "varprojlim" => Ok(Expr::ArrowAccent {
                kind: ArrowAccentKind::Left,
                under: true,
                content: Box::new(Expr::FunctionName("lim".to_string())),
            }),
            "varliminf" => Ok(Expr::BarTemplate {
                kind: BarTemplateKind::Under,
                content: Box::new(Expr::FunctionName("lim".to_string())),
            }),
            "varlimsup" => Ok(Expr::BarTemplate {
                kind: BarTemplateKind::Over,
                content: Box::new(Expr::FunctionName("lim".to_string())),
            }),
            "bmod" => Ok(Expr::Sequence(vec![
                Expr::Space(0x02),
                Expr::FunctionName("mod".to_string()),
                Expr::Space(0x02),
            ])),
            "mod" => Ok(Expr::Sequence(vec![
                Expr::Space(0x05),
                Expr::FunctionName("mod".to_string()),
                // Probe bytes show MathType uses a narrower trailing space after \mod.
                Expr::Space(0x02),
            ])),
            "not" => self.parse_not_relation(),
            "pmod" | "pod" => {
                let argument = self.parse_required_group_or_atom("modulo argument")?;
                Ok(modulo_parenthesized_expr(command.as_str(), argument))
            }
            "ket" | "Ket" => Ok(raw_prefix_expr(
                command.as_str(),
                self.parse_required_group("ket content")?,
            )),
            "VERT" => Ok(Expr::RawTex("\\VERT".to_string())),
            "bra" | "Bra" => Ok(raw_prefix_expr(
                command.as_str(),
                self.parse_required_group("bra content")?,
            )),
            "braket" | "Braket" => Ok(raw_prefix_expr(
                command.as_str(),
                self.parse_required_group("braket content")?,
            )),
            "set" | "Set" => Ok(raw_prefix_expr(
                command.as_str(),
                self.parse_required_group("set content")?,
            )),
            "rm" => Ok(Expr::Font {
                // MathType applies the legacy switch form like \mathrm, so the
                // whole switch span must render with fnTEXT rather than default
                // variable/number typefaces.
                kind: FontKind::RomanText,
                content: Box::new(self.parse_switch_content("rm content")?),
            }),
            "it" => Ok(self.parse_switch_content("it content")?),
            "mathrm" | "textrm" => Ok(Expr::Font {
                kind: FontKind::RomanText,
                content: Box::new(self.parse_required_group("roman content")?),
            }),
            "mathnormal" | "textnormal" | "textup" | "textmd" => Ok(raw_prefix_expr(
                command.as_str(),
                self.parse_required_group("roman content")?,
            )),
            "mathit" | "textit" | "emph" => self.parse_required_group("italic content"),
            "bf" => Ok(Expr::Font {
                kind: FontKind::Bold,
                content: Box::new(self.parse_switch_content("bf content")?),
            }),
            "sf" => Ok(raw_prefix_expr(
                "sf",
                self.parse_switch_content("sf content")?,
            )),
            "mathbf" | "textbf" | "boldsymbol" | "bold" => Ok(Expr::Font {
                kind: FontKind::Bold,
                content: Box::new(self.parse_required_group("mathbf content")?),
            }),
            "bm" => Ok(raw_prefix_expr(
                "bm",
                self.parse_required_group("bm content")?,
            )),
            "pmb" => {
                self.skip_ws();
                if self.peek() == Some('{') {
                    Ok(raw_prefix_expr(
                        "pmb",
                        self.parse_required_group("pmb content")?,
                    ))
                } else {
                    Ok(Expr::RawTex("\\pmb".to_string()))
                }
            }
            "cal" => self.parse_switch_content("cal content"),
            "mathcal" => Ok(Expr::Font {
                kind: FontKind::MathCal,
                content: Box::new(self.parse_required_group("mathcal content")?),
            }),
            "mathsf" => Ok(Expr::Font {
                kind: FontKind::MathSf,
                content: Box::new(self.parse_required_group("mathsf content")?),
            }),
            "textsf" => Ok(raw_prefix_expr(
                "textsf",
                self.parse_required_group("textsf content")?,
            )),
            "mathtt" => Ok(raw_prefix_expr(
                "mathtt",
                self.parse_required_group("mathtt content")?,
            )),
            "texttt" => Ok(Expr::Font {
                kind: FontKind::TypewriterText,
                content: Box::new(self.parse_required_group("texttt content")?),
            }),
            "tt" => Ok(raw_prefix_expr(
                "tt",
                self.parse_switch_content("tt content")?,
            )),
            "mathbb" | "Bbb" => Ok(Expr::Font {
                kind: FontKind::MathBb,
                content: Box::new(self.parse_required_group("mathbb content")?),
            }),
            "Complex" | "C" | "cnums" => Ok(blackboard_letter('C')),
            "Naturals" | "N" | "natnums" => Ok(blackboard_letter('N')),
            "Q" | "Rational" | "Rationals" => Ok(blackboard_letter('Q')),
            "R" | "Reals" | "reals" => Ok(blackboard_letter('R')),
            "Z" | "Integers" => Ok(blackboard_letter('Z')),
            "mathscr" => Ok(Expr::Font {
                kind: FontKind::MathScr,
                content: Box::new(self.parse_required_group("mathscr content")?),
            }),
            "mathfrak" => Ok(Expr::Font {
                kind: FontKind::MathFrak,
                content: Box::new(self.parse_required_group("mathfrak content")?),
            }),
            "bar" => Ok(Expr::Accent {
                kind: AccentKind::Bar,
                content: Box::new(self.parse_required_group("bar content")?),
            }),
            "overline" => Ok(Expr::BarTemplate {
                kind: BarTemplateKind::Over,
                content: Box::new(self.parse_required_group("overline content")?),
            }),
            "underline" => Ok(Expr::BarTemplate {
                kind: BarTemplateKind::Under,
                content: Box::new(self.parse_required_group("underline content")?),
            }),
            // MathType preserves the command token for \underbar instead of emitting
            // the native underbar template used by \underline.
            "underbar" => self.parse_raw_prefix_group_command("underbar content", "underbar"),
            "cancel" => Ok(Expr::Strike {
                kind: StrikeKind::Up,
                content: Box::new(self.parse_required_group("cancel content")?),
            }),
            "bcancel" => Ok(Expr::Strike {
                kind: StrikeKind::Down,
                content: Box::new(self.parse_required_group("bcancel content")?),
            }),
            "xcancel" => Ok(Expr::Strike {
                kind: StrikeKind::Both,
                content: Box::new(self.parse_required_group("xcancel content")?),
            }),
            "sout" => Ok(Expr::Strike {
                kind: StrikeKind::Horizontal,
                content: Box::new(self.parse_required_group("sout content")?),
            }),
            "hat" => Ok(Expr::Accent {
                kind: AccentKind::Hat,
                content: Box::new(self.parse_required_group("hat content")?),
            }),
            "widehat" => Ok(Expr::Accent {
                kind: AccentKind::WideHat,
                content: Box::new(self.parse_required_group("widehat content")?),
            }),
            "breve" | "u" => Ok(Expr::Accent {
                kind: AccentKind::Breve,
                content: Box::new(self.parse_required_group("breve content")?),
            }),
            "dot" => Ok(Expr::Accent {
                kind: AccentKind::Dot,
                content: Box::new(self.parse_required_group("dot content")?),
            }),
            "ddot" => Ok(Expr::Accent {
                kind: AccentKind::Ddot,
                content: Box::new(self.parse_required_group("ddot content")?),
            }),
            "dddot" => Ok(Expr::Accent {
                kind: AccentKind::Dddot,
                content: Box::new(self.parse_required_group("dddot content")?),
            }),
            "ddddot" => Ok(Expr::Accent {
                kind: AccentKind::Ddddot,
                content: Box::new(self.parse_required_group("ddddot content")?),
            }),
            "tilde" => Ok(Expr::Accent {
                kind: AccentKind::Tilde,
                content: Box::new(self.parse_required_group("tilde content")?),
            }),
            // MathType preserves the command token for \utilde instead of
            // lowering it to native under-tilde embellishment records.
            "utilde" => self.parse_raw_prefix_group_command("utilde content", "utilde"),
            "acute" => Ok(Expr::Accent {
                kind: AccentKind::Acute,
                content: Box::new(self.parse_required_group("acute content")?),
            }),
            "grave" => Ok(Expr::Accent {
                kind: AccentKind::Grave,
                content: Box::new(self.parse_required_group("grave content")?),
            }),
            "check" | "v" => Ok(Expr::Accent {
                kind: AccentKind::Check,
                content: Box::new(self.parse_required_group("check content")?),
            }),
            "widecheck" => self.parse_raw_prefix_group_command("widecheck content", "widecheck"),
            "Overrightarrow" => Ok(raw_prefix_expr(
                command.as_str(),
                self.parse_required_group("vector arrow content")?,
            )),
            "vec" | "overrightarrow" => Ok(Expr::ArrowAccent {
                kind: ArrowAccentKind::Right,
                under: false,
                content: Box::new(self.parse_required_group("vector arrow content")?),
            }),
            "overleftarrow" => Ok(Expr::ArrowAccent {
                kind: ArrowAccentKind::Left,
                under: false,
                content: Box::new(self.parse_required_group("overleftarrow content")?),
            }),
            "underleftarrow" | "underrightarrow" => Ok(raw_prefix_expr(
                command.as_str(),
                self.parse_required_group("under-arrow content")?,
            )),
            "overleftrightarrow" => Ok(Expr::ArrowAccent {
                kind: ArrowAccentKind::LeftRight,
                under: false,
                content: Box::new(self.parse_required_group("overleftrightarrow content")?),
            }),
            "underleftrightarrow" => Ok(raw_prefix_expr(
                command.as_str(),
                self.parse_required_group("underleftrightarrow content")?,
            )),
            "overleftharpoon" | "overrightharpoon" => Ok(raw_prefix_expr(
                command.as_str(),
                self.parse_required_group("harpoon accent content")?,
            )),
            "overbrace" => Ok(Expr::Brace {
                kind: BraceKind::Over,
                content: Box::new(self.parse_required_group("overbrace content")?),
                annotation: None,
            }),
            "underbrace" => Ok(Expr::Brace {
                kind: BraceKind::Under,
                content: Box::new(self.parse_required_group("underbrace content")?),
                annotation: None,
            }),
            "overbracket" | "underbracket" => {
                self.parse_raw_prefix_group_command("bracket content", command.as_str())
            }
            "stackrel" | "overset" => {
                let upper = self.parse_required_group("stackrel upper")?;
                let lower = self.parse_required_group("stackrel lower")?;
                Ok(Expr::Stackrel {
                    upper: Box::new(upper),
                    lower: Box::new(lower),
                })
            }
            "underset" => {
                let lower = self.parse_required_group("underset lower")?;
                let base = self.parse_required_group("underset base")?;
                Ok(Expr::Underset {
                    lower: Box::new(lower),
                    base: Box::new(base),
                })
            }
            "mathclap" | "mathllap" | "mathrlap" => self.parse_overlap_wrapper(command.as_str()),
            "vcenter" => {
                let content = self.parse_required_group("vcenter content")?;
                Ok(raw_prefix_expr("vcenter", content))
            }
            "hspace" => self.parse_hspace_content(),
            "hline" | "hdashline" => Ok(Expr::Sequence(Vec::new())),
            "cline" => self.parse_cline_content(),
            "phase" => Ok(raw_prefix_expr(
                command.as_str(),
                self.parse_required_group("phase angle")?,
            )),
            _ if spacing_command_width(&command).is_some() => {
                Ok(Expr::Space(spacing_command_width(&command).unwrap()))
            }
            _ if raw_hybrid_xarrow_command(&command) => {
                let under = self.parse_optional_bracket_group()?;
                let label = self.parse_required_group("arrow label")?;
                let mut args = vec![label];
                if let Some(under) = under {
                    args.push(under);
                }
                Ok(raw_prefix_sequence(command.as_str(), args))
            }
            _ if xarrow_command_kind(&command).is_some() => {
                let under = self.parse_optional_bracket_group()?;
                let label = self.parse_required_group("arrow label")?;
                Ok(Expr::XArrow {
                    kind: xarrow_command_kind(&command).unwrap(),
                    label: Box::new(label),
                    under: under.map(Box::new),
                })
            }
            _ if big_symbol_command_to_char(&command).is_some() => Ok(Expr::BigSymbol(
                big_symbol_command_to_char(&command).unwrap(),
            )),
            _ if sum_operator_command_to_char(&command).is_some() => Ok(Expr::SumOperatorSymbol(
                sum_operator_command_to_char(&command).unwrap(),
            )),
            _ if command_specific_to_char(&command).is_some() => Ok(Expr::CommandSymbol {
                ch: command_specific_to_char(&command).unwrap(),
                command: command.clone(),
            }),
            _ if delimiter_command_char(&command).is_some() => {
                Ok(Expr::Char(delimiter_command_char(&command).unwrap()))
            }
            _ if command_to_sequence(&command).is_some() => Ok(Expr::Sequence(
                command_to_sequence(&command)
                    .unwrap()
                    .iter()
                    .map(|ch| Expr::Char(*ch))
                    .collect(),
            )),
            _ if command_to_text(&command).is_some() => {
                Ok(Expr::Text(command_to_text(&command).unwrap().to_string()))
            }
            _ if command_to_char(&command).is_some() => {
                Ok(Expr::Char(command_to_char(&command).unwrap()))
            }
            "" => {
                let ch = self
                    .next()
                    .ok_or_else(|| "dangling backslash".to_string())?;
                if let Some(width) = escaped_single_char_space(ch) {
                    return Ok(Expr::Space(width));
                }
                if ch.is_whitespace() {
                    // MathType's escaped control-space uses the narrower fnSPACE 0xef04 slot,
                    // not the regular `\space` / `\nobreakspace` width.
                    return Ok(Expr::Space(0x04));
                }
                if ch == '&' {
                    // MathType TeX Input keeps `\&` on the raw-text path instead of converting it
                    // into a native punctuation CHAR record.
                    return Ok(with_leading_raw_space(
                        Expr::RawTex("\\&".to_string()),
                        had_leading_ws,
                    ));
                }
                if let Some(text_char) = escaped_single_char_math_char(ch) {
                    return Ok(Expr::Char(text_char));
                }
                if ch == '|' {
                    return Ok(Expr::Char('\u{2016}'));
                }
                Ok(Expr::Char(ch))
            }
            _ => self.parse_unsupported_command(command),
        }?;
        Ok(with_leading_raw_space(expr, had_leading_ws))
    }
}
