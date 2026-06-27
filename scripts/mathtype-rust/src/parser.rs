use crate::ast::*;
use crate::generated::char_tables::{
    BIG_SYMBOL_COMMAND_CHARS, COMMAND_SPECIFIC_CHARS, DELIMITER_COMMAND_CHARS,
    SUM_OPERATOR_COMMAND_CHARS, TEX_COMMAND_CHARS, TEX_COMMAND_SEQUENCES, TEX_COMMAND_TEXTS,
};
use crate::generated::raw_text_tables::{literal_raw_text_override, LiteralOverrideFragment};
use crate::mathtype_ansi::encode_mathtype_text;
use crate::raw_fallback::should_force_raw_simple_command;
use std::collections::HashMap;

#[path = "parser/text_mode.rs"]
mod text_mode;

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
            | "sec" | "sh" | "sin" | "sinh" | "sup" | "tan" | "tanh" | "tg" | "th"
            | "Pr" => Ok(Expr::FunctionName(command)),
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
            "sf" => Ok(raw_prefix_expr("sf", self.parse_switch_content("sf content")?)),
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
            "mathclap" | "mathllap" | "mathrlap" => {
                self.parse_overlap_wrapper(command.as_str())
            },
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
            },
            _ if raw_hybrid_xarrow_command(&command) => {
                let under = self.parse_optional_bracket_group()?;
                let label = self.parse_required_group("arrow label")?;
                let mut args = vec![label];
                if let Some(under) = under {
                    args.push(under);
                }
                Ok(raw_prefix_sequence(command.as_str(), args))
            },
            _ if xarrow_command_kind(&command).is_some() => {
                let under = self.parse_optional_bracket_group()?;
                let label = self.parse_required_group("arrow label")?;
                Ok(Expr::XArrow {
                    kind: xarrow_command_kind(&command).unwrap(),
                    label: Box::new(label),
                    under: under.map(Box::new),
                })
            },
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
            },
            _ if command_to_sequence(&command).is_some() => Ok(Expr::Sequence(
                command_to_sequence(&command)
                    .unwrap()
                    .iter()
                    .map(|ch| Expr::Char(*ch))
                    .collect(),
            )),
            _ if command_to_text(&command).is_some() => {
                Ok(Expr::Text(command_to_text(&command).unwrap().to_string()))
            },
            _ if command_to_char(&command).is_some() => {
                Ok(Expr::Char(command_to_char(&command).unwrap()))
            },
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
            },
            _ => self.parse_unsupported_command(command),
        }?;
        Ok(with_leading_raw_space(expr, had_leading_ws))
    }

    /// Parse one braced command whose MathType fallback keeps the command name raw.
    fn parse_raw_prefix_group_command(&mut self, label: &str, command: &str) -> Result<Expr, String> {
        Ok(raw_prefix_expr(command, self.parse_required_group(label)?))
    }

    /// Preserve MathType's overlap wrappers as one raw prefix followed by visible groups.
    fn parse_overlap_wrapper(&mut self, command: &str) -> Result<Expr, String> {
        let first = self.parse_required_group("overlap content")?;
        let mut args = Vec::new();
        // Flatten the visible group even when there is only one argument so the
        // writer emits the same single-line slot structure MathType uses for
        // mathclap/mathllap/mathrlap script operands.
        push_visible_items(&mut args, first);
        // MathType keeps a second adjacent braced group visible in the same raw-prefix run,
        // as in mathrlap{,/}{=}.
        self.skip_ws();
        if self.peek() == Some('{') {
            push_visible_items(&mut args, self.parse_required_group("overlap trailing content")?);
        }
        Ok(raw_prefix_sequence(command, args))
    }

    /// Parse one braced command whose MathType fallback keeps only the leading backslashes raw.
    fn parse_hybrid_backslash_group_command(
        &mut self,
        label: &str,
        command_text: &str,
    ) -> Result<Expr, String> {
        let content = self.parse_required_group(label)?;
        self.parse_hybrid_backslash_command(command_text, vec![content])
    }

    /// Parse one two-argument command whose MathType fallback keeps only the command name raw.
    fn parse_raw_prefix_two_groups_command(
        &mut self,
        first_label: &str,
        second_label: &str,
        command: &str,
    ) -> Result<Expr, String> {
        let first = self.parse_required_group(first_label)?;
        let second = self.parse_required_group(second_label)?;
        Ok(raw_prefix_sequence(command, vec![first, second]))
    }

    /// Parse one hybrid MathType fallback that stores only the leading backslashes as raw text.
    fn parse_hybrid_backslash_command(
        &self,
        command_text: &str,
        args: Vec<Expr>,
    ) -> Result<Expr, String> {
        let mut items = Vec::with_capacity(args.len() + 2);
        items.push(Expr::RawTex("\\\\".to_string()));
        items.push(self.parse_visible_wrapper_text(command_text)?);
        items.extend(args);
        Ok(Expr::Sequence(items))
    }

    /// Preserve unsupported TeX primitive assignments as one raw text run.
    fn consume_raw_primitive(&mut self, command: String) -> String {
        let mut raw = format!("\\{command}");
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                break;
            }
            raw.push(ch);
            self.pos += 1;
        }
        raw
    }

    /// Parse \utilde when it can be represented by documented character embellishments.
    fn parse_under_tilde_content(&mut self) -> Result<Expr, String> {
        let raw = self.parse_raw_group("utilde content")?;
        if !is_simple_embellishment_run(&raw) {
            return Ok(Expr::RawTex(format!("\\utilde{{{raw}}}")));
        }
        Ok(Expr::Accent {
            kind: AccentKind::UnderTilde,
            content: Box::new(Expr::Sequence(raw.chars().map(Expr::Char).collect())),
        })
    }

    /// Parse a narrow \def/\gdef subset used by Supported Functions examples.
    fn parse_macro_definition(&mut self, command: &str) -> Result<Expr, String> {
        self.skip_ws();
        let Some(name) = self.parse_macro_name()? else {
            return Ok(Expr::RawTex(format!("\\{command}")));
        };
        let params = self.parse_macro_parameter_count()?;
        let replacement = self.parse_raw_group("macro replacement")?;
        self.macros.insert(
            name.clone(),
            MacroDefinition {
                params,
                replacement: replacement.clone(),
                render_mode: MacroRenderMode::RawOnly,
            },
        );
        self.render_macro_definition(command, &name, params, &replacement)
    }

    /// Expand a previously defined simple macro command, if one is in scope.
    fn expand_macro_command(&mut self, command: &str) -> Result<Option<Expr>, String> {
        let Some(definition) = self.macros.get(command).cloned() else {
            return Ok(None);
        };
        match definition.render_mode {
            MacroRenderMode::RawOnly => {
                self.render_raw_macro_invocation(command, definition.params)
            }
        }
    }

    /// Parse a replacement string with the current macro scope and depth limit.
    fn parse_macro_replacement(&self, replacement: &str) -> Result<Expr, String> {
        let mut parser = Parser {
            chars: replacement.chars().collect(),
            pos: 0,
            macros: self.macros.clone(),
            expansion_depth: self.expansion_depth + 1,
            pending_raw_ws: false,
        };
        let expr = parser.parse_sequence(None)?;
        parser.skip_ws();
        if parser.pos != parser.chars.len() {
            return Err(format!(
                "unexpected character in macro replacement {:?}",
                parser.peek()
            ));
        }
        Ok(expr)
    }

    /// Render one raw-only macro definition the way MathType TeX Input exposes it.
    fn render_macro_definition(
        &self,
        command: &str,
        name: &str,
        params: usize,
        replacement: &str,
    ) -> Result<Expr, String> {
        let mut args = Vec::new();
        let raw_prefix = match params {
            0 => format!("\\{command}\\{name}"),
            1 => {
                args.push(Expr::Char('1'));
                format!("\\{command}\\{name}#")
            }
            _ => return Ok(Expr::RawTex(format!("\\{command}"))),
        };
        args.extend(render_raw_only_macro_replacement(replacement, self)?);
        Ok(raw_prefix_sequence_with_raw(raw_prefix, args))
    }

    /// Render one raw-only macro invocation without expanding it into native MathType structures.
    fn render_raw_macro_invocation(
        &mut self,
        command: &str,
        params: usize,
    ) -> Result<Option<Expr>, String> {
        match params {
            0 => Ok(Some(Expr::RawTex(format!("\\{command}")))),
            1 => {
                let argument = match self.parse_raw_group("macro argument") {
                    Ok(argument) => argument,
                    Err(_) => return Ok(Some(Expr::RawTex(format!("\\{command}")))),
                };
                Ok(Some(raw_prefix_expr(
                    command,
                    self.parse_visible_wrapper_text(&argument)?,
                )))
            }
            _ => Ok(Some(Expr::RawTex(format!("\\{command}")))),
        }
    }

    /// Parse the control-word name after \def or \gdef.
    fn parse_macro_name(&mut self) -> Result<Option<String>, String> {
        self.skip_ws();
        if self.peek() != Some('\\') {
            return Ok(None);
        }
        self.pos += 1;
        let start = self.pos;
        while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
            self.pos += 1;
        }
        if self.pos == start {
            return Err("expected macro control-word name".to_string());
        }
        Ok(Some(self.chars[start..self.pos].iter().collect()))
    }

    /// Parse the supported macro signatures: no parameters or one `#1` parameter.
    fn parse_macro_parameter_count(&mut self) -> Result<usize, String> {
        self.skip_ws();
        if self.peek() != Some('#') {
            return Ok(0);
        }
        self.pos += 1;
        if self.peek() != Some('1') {
            return Err("only one-argument macro definitions are supported".to_string());
        }
        self.pos += 1;
        Ok(1)
    }

    /// Parse \middle followed by a delimiter inside \left...\right content.
    fn parse_middle_delimiter(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if self.pos >= self.chars.len()
            || self.starts_command("right")
            || self.starts_command("end")
            || self.starts_row_separator()
            || matches!(self.peek(), Some('}' | '&'))
        {
            return self.parse_unsupported_command("middle".to_string());
        }
        let delimiter = self.parse_delimiter_char("middle delimiter")?;
        Ok(if delimiter == '.' {
            Expr::Sequence(Vec::new())
        } else {
            Expr::Char(delimiter)
        })
    }

    /// Preserve MathType''s hybrid \char" form: raw command prefix plus visible hex digits.
    fn parse_char_code(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        let start = self.pos;
        if self.peek() != Some('"') {
            return self.parse_unsupported_command("char".to_string());
        }
        self.pos += 1;
        let digits_start = self.pos;
        while self.peek().is_some_and(|ch| ch.is_ascii_hexdigit()) {
            self.pos += 1;
        }
        if self.pos == digits_start {
            self.pos = start;
            return self.parse_unsupported_command("char".to_string());
        }
        let digits = self.chars[digits_start..self.pos]
            .iter()
            .collect::<String>();
        u32::from_str_radix(&digits, 16)
            .map_err(|err| format!("invalid \\char hex code {digits}: {err}"))?;
        Ok(raw_prefix_sequence_with_raw(
            "\\char\"".to_string(),
            vec![self.parse_visible_wrapper_text(&digits)?],
        ))
    }

    /// Preserve MathType's explicit display/text binomial wrappers instead of collapsing them.
    fn parse_styled_binom(&mut self, kind: StyleKind) -> Result<Expr, String> {
        let upper = self.parse_required_group("binomial upper")?;
        let lower = self.parse_required_group("binomial lower")?;
        Ok(Expr::Style {
            kind,
            content: Box::new(Expr::Pile {
                kind: PileKind::Binom,
                upper: Box::new(upper),
                lower: Box::new(lower),
            }),
        })
    }

    /// Preserve MathType's raw \smash prefix while keeping any optional position and content visible.
    fn parse_smash_content(&mut self) -> Result<Expr, String> {
        let position = self.parse_optional_raw_bracket_group()?;
        let content = self.parse_required_group("smash content")?;
        let mut args = Vec::with_capacity(2);
        if let Some(position) = position {
            args.push(self.parse_visible_wrapper_text(&format!("[{position}]"))?);
        }
        args.push(content);
        Ok(raw_prefix_sequence_with_raw("\\smash".to_string(), args))
    }

    /// Parse TeX's delimiter-based \verb literal as visible typewriter text.
    fn parse_verb_literal(&mut self) -> Result<Expr, String> {
        let Some(delimiter) = self.next() else {
            return Ok(Expr::RawTex("\\verb".to_string()));
        };
        if delimiter.is_whitespace() {
            return Ok(Expr::RawTex("\\verb".to_string()));
        }
        let start = self.pos;
        while self.peek().is_some_and(|ch| ch != delimiter) {
            self.pos += 1;
        }
        if self.peek() != Some(delimiter) {
            self.pos = start;
            return Ok(Expr::RawTex("\\verb".to_string()));
        }
        let content = self.chars[start..self.pos].iter().collect::<String>();
        self.pos += 1;
        let visible =
            self.parse_visible_wrapper_text(&format!("{delimiter}{content}{delimiter}"))?;
        Ok(raw_prefix_expr("verb", visible))
    }

    /// Parse `\not` as a relation overlay instead of collapsing it to Unicode.
    ///
    /// MathType TeX Input keeps the base relation glyph and adds the same `embNOT`
    /// decoration in MTEF, so we preserve the inner relation expression here.
    fn parse_not_relation(&mut self) -> Result<Expr, String> {
        let relation = self.parse_required_group_or_atom("not relation")?;
        if relation_char(&relation).is_none() {
            return Ok(raw_prefix_expr("not", relation));
        }
        Ok(Expr::NotRelation(Box::new(relation)))
    }

    /// Preserve MathType TeX Input fallback for unsupported control words.
    fn parse_unsupported_command(&mut self, command: String) -> Result<Expr, String> {
        let mut raw = format!("\\{command}");
        let consumed_ws = self.consume_ws();
        if self.peek() == Some('{') {
            self.pos += 1;
            let argument = self.parse_sequence(Some('}'))?;
            Ok(Expr::Sequence(vec![Expr::RawTex(raw), argument]))
        } else {
            if consumed_ws && raw_command_preserves_trailing_space(&command) {
                raw.push(' ');
            }
            Ok(Expr::RawTex(raw))
        }
    }

    /// Parse one atom, its scripts, and any operand required by big operators.
    fn parse_complete_atom(&mut self) -> Result<Expr, String> {
        let mut atom = self.parse_atom_with_scripts()?;
        if matches!(atom, Expr::BigOp { body: None, .. }) && !self.big_op_operand_stops() {
            let body = self.parse_big_op_operand()?;
            if let Expr::BigOp {
                body: body_slot, ..
            } = &mut atom
            {
                *body_slot = Some(Box::new(body));
            }
        } else if matches!(
            atom,
            Expr::Integral { .. } | Expr::IntegralOp { body: None, .. }
        ) && !self.big_op_operand_stops()
        {
            let body = self.parse_big_op_operand()?;
            match &mut atom {
                Expr::Integral { kind } => {
                    atom = Expr::IntegralOp {
                        kind: *kind,
                        lower: None,
                        upper: None,
                        body: Some(Box::new(body)),
                    };
                }
                Expr::IntegralOp {
                    body: body_slot, ..
                } => {
                    *body_slot = Some(Box::new(body));
                }
                _ => {}
            }
        }
        Ok(atom)
    }

    /// Parse an atom followed by optional subscript/superscript records.
    fn parse_atom_with_scripts(&mut self) -> Result<Expr, String> {
        // MathType accepts leading `_` / `^` as scripts on an empty base instead
        // of treating the underscore/caret as a visible atom.
        let mut atom = if matches!(self.peek(), Some('_' | '^')) {
            Expr::Sequence(Vec::new())
        } else {
            self.parse_atom()?
        };
        let mut limit_modifier = None;
        loop {
            let consumed_ws = self.consume_ws();
            if let Some(modifier) = self.consume_limits_modifier() {
                limit_modifier = Some(modifier);
                continue;
            }
            match self.peek() {
                Some('_') => {
                    self.pos += 1;
                    let sub = self.parse_script_arg()?;
                    atom = merge_script(atom, Some(sub), None, limit_modifier.take());
                }
                Some('^') => {
                    self.pos += 1;
                    if let Some(raw) = self.parse_raw_prime_script_group()? {
                        atom = raw_superscript_suffix_expr(atom, &raw);
                        continue;
                    }
                    let sup = self.parse_script_arg()?;
                    atom = merge_script(atom, None, Some(sup), limit_modifier.take());
                }
                _ => {
                    if consumed_ws {
                        // MathType keeps this source-space inside the next raw-text fallback run.
                        self.pending_raw_ws = true;
                    }
                    break;
                }
            }
        }
        Ok(atom)
    }

    /// Parse the term MathType places in the first slot of a big-op template.
    fn parse_big_op_operand(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if self.big_op_operand_stops() {
            return Err("expected big-operator operand".to_string());
        }
        self.parse_complete_atom()
    }

    /// Stop a big-op operand at top-level separators; nested groups parse themselves.
    fn big_op_operand_stops(&self) -> bool {
        self.pos >= self.chars.len()
            || self.starts_command("right")
            || self.starts_command("end")
            || self.starts_row_separator()
            || matches!(self.peek(), Some('}' | ',' | '+' | '-' | '=' | '&'))
    }

    /// Parse a dynamic delimiter body until the paired \right command.
    fn parse_sequence_until_right(&mut self) -> Result<Expr, String> {
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.pos >= self.chars.len() {
                return Err("unterminated \\left...\\right group".to_string());
            }
            if self.starts_command("right") {
                break;
            }
            if let Some(infix) = self.consume_infix_command() {
                let left = Expr::Sequence(items);
                let right = self.parse_sequence_until_right()?;
                return Ok(infix_expr(infix, left, right));
            }
            let atom = self.parse_complete_atom()?;
            if expr_is_empty_sequence(&atom) {
                continue;
            }
            items.push(atom);
        }
        Ok(Expr::Sequence(items))
    }

    /// Parse rows and columns for the supported \begin...\end environments.
    fn parse_environment(&mut self, name: &str) -> Result<Expr, String> {
        let base_name = name.strip_suffix('*').unwrap_or(name);
        if name.ends_with('*') {
            // mathtools matrix* variants accept an optional alignment specifier.
            let _ = self.parse_optional_bracket_group()?;
        }
        let parsed_rows;
        let kind = match base_name {
            "equation" => return self.parse_wrapper_environment(name),
            "split" => EnvironmentKind::Split,
            "align" => EnvironmentKind::Align,
            "alignat" => {
                self.parse_raw_group("alignat column count")?;
                EnvironmentKind::AlignAt
            }
            "aligned" => EnvironmentKind::Aligned,
            "alignedat" => {
                self.parse_raw_group("alignedat column count")?;
                EnvironmentKind::AlignedAt
            }
            "gather" => EnvironmentKind::Gather,
            "gathered" => EnvironmentKind::Gathered,
            "cases" | "dcases" => EnvironmentKind::Cases,
            "rcases" | "drcases" => EnvironmentKind::RightCases,
            "array" => {
                // Preserve row-rule control words such as \hdashline so the writer can
                // reproduce MathType's mixed native/raw array layout instead of flattening
                // the whole environment into one raw fallback run.
                self.parse_raw_group("array column specifier")?;
                parsed_rows = self.parse_array_rows(name)?;
                return Ok(Expr::Environment {
                    kind: EnvironmentKind::Array,
                    rows: parsed_rows.rows,
                    trivia: EnvironmentTrivia {
                        row_leading: parsed_rows.row_leading,
                        separator_leading: parsed_rows.separator_leading,
                        end_leading: parsed_rows.end_leading,
                    },
                });
            }
            "subarray" => {
                // Preserve the column specifier so the writer can reproduce MathType's
                // mixed raw/native rendering for big-operator limits that use subarray.
                let column_spec = self.parse_raw_group("subarray column specifier")?;
                parsed_rows = self.parse_environment_rows(name)?;
                return Ok(Expr::Subarray {
                    column_spec,
                    rows: parsed_rows.rows,
                });
            }
            "matrix" => {
                parsed_rows = self.parse_environment_rows(name)?;
                return Ok(Expr::Matrix {
                    kind: MatrixKind::Plain,
                    rows: parsed_rows.rows,
                })
            }
            // MathType writes smallmatrix as a plain matrix entered under an explicit script-size record.
            "smallmatrix" => {
                parsed_rows = self.parse_environment_rows(name)?;
                return Ok(Expr::Style {
                    kind: StyleKind::Script,
                    content: Box::new(Expr::Matrix {
                        kind: MatrixKind::Small,
                        rows: parsed_rows.rows,
                    }),
                })
            }
            "pmatrix" => {
                parsed_rows = self.parse_environment_rows(name)?;
                return Ok(Expr::Matrix {
                    kind: MatrixKind::Parenthesized,
                    rows: parsed_rows.rows,
                })
            }
            "bmatrix" => {
                parsed_rows = self.parse_environment_rows(name)?;
                return Ok(Expr::Matrix {
                    kind: MatrixKind::Bracketed,
                    rows: parsed_rows.rows,
                })
            }
            "Bmatrix" => {
                parsed_rows = self.parse_environment_rows(name)?;
                return Ok(Expr::Matrix {
                    kind: MatrixKind::Braced,
                    rows: parsed_rows.rows,
                })
            }
            "vmatrix" => {
                parsed_rows = self.parse_environment_rows(name)?;
                return Ok(Expr::Matrix {
                    kind: MatrixKind::Barred,
                    rows: parsed_rows.rows,
                })
            }
            "Vmatrix" => {
                parsed_rows = self.parse_environment_rows(name)?;
                return Ok(Expr::Matrix {
                    kind: MatrixKind::DoubleBarred,
                    rows: parsed_rows.rows,
                })
            }
            _ => return self.parse_unsupported_environment(name),
        };
        parsed_rows = self.parse_environment_rows(name)?;
        Ok(Expr::Environment {
            kind,
            rows: parsed_rows.rows,
            trivia: EnvironmentTrivia {
                row_leading: parsed_rows.row_leading,
                separator_leading: parsed_rows.separator_leading,
                end_leading: parsed_rows.end_leading,
            },
        })
    }

    /// Parse transparent wrappers such as equation/equation* around real math content.
    fn parse_wrapper_environment(&mut self, name: &str) -> Result<Expr, String> {
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.pos >= self.chars.len() {
                return Err(format!("unterminated \\begin{{{name}}} environment"));
            }
            if self.starts_end_environment(name) {
                self.consume_end_environment(name)?;
                break;
            }
            items.push(self.parse_complete_atom()?);
        }
        Ok(Expr::Sequence(items))
    }

    /// Preserve an unsupported environment using the probe-backed fallback shape.
    fn parse_unsupported_environment(&mut self, name: &str) -> Result<Expr, String> {
        if unsupported_environment_uses_hybrid_begin_end(name) {
            let body = self.parse_hybrid_unsupported_environment_body(name)?;
            return Ok(Expr::Sequence(vec![
                Expr::RawTex("\\begin".to_string()),
                visible_text_sequence(name),
                body,
                Expr::RawTex("\\end".to_string()),
                visible_text_sequence(name),
            ]));
        }
        let mut raw = format!("\\begin{{{name}}}");
        loop {
            if self.pos >= self.chars.len() {
                return Err(format!("unterminated unsupported environment: {name}"));
            }
            if self.starts_end_environment(name) {
                raw.push_str(&format!("\\end{{{name}}}"));
                self.consume_end_environment(name)?;
                break;
            }
            let ch = self
                .next()
                .ok_or_else(|| format!("unterminated unsupported environment: {name}"))?;
            raw.push(ch);
        }
        Ok(Expr::RawTex(raw))
    }

    /// Parse probe-known unsupported environments whose begin/end stay raw while
    /// MathType still tokenizes the interior as visible characters.
    fn parse_hybrid_unsupported_environment_body(&mut self, name: &str) -> Result<Expr, String> {
        let mut items = Vec::new();
        loop {
            let _ = self.consume_raw_whitespace();
            if self.pos >= self.chars.len() {
                return Err(format!("unterminated unsupported environment: {name}"));
            }
            if self.starts_end_environment(name) {
                self.consume_end_environment(name)?;
                break;
            }
            if self.consume_row_separator() {
                continue;
            }
            let atom = self.parse_complete_atom()?;
            if expr_is_empty_sequence(&atom) {
                continue;
            }
            items.push(atom);
        }
        Ok(Expr::Sequence(items))
    }
    /// Parse an environment into rows split by & and \\ separators.
    fn parse_environment_rows(&mut self, name: &str) -> Result<ParsedEnvironmentRows, String> {
        let mut rows = Vec::new();
        let mut row_leading = Vec::new();
        let mut separator_leading = Vec::new();
        loop {
            let leading_ws = normalize_environment_fallback_whitespace(&self.consume_raw_whitespace());
            if self.starts_command("end") {
                self.consume_end_environment(name)?;
                return Ok(ParsedEnvironmentRows {
                    rows,
                    row_leading,
                    separator_leading,
                    end_leading: leading_ws,
                });
            }
            row_leading.push(leading_ws);
            let mut cells = Vec::new();
            let mut row_separator_prefixes = Vec::new();
            loop {
                let cell = self.parse_sequence_until_environment_stop(name)?;
                cells.push(cell);
                let trailing_ws =
                    normalize_environment_fallback_whitespace(&self.consume_raw_whitespace());
                if self.peek() == Some('&') {
                    // When MathType fallback preserves a literal space before `&`,
                    // recover it from the source if the parser's stop logic left
                    // the separator-adjacent trivia behind the current cursor.
                    let separator_prefix = if trailing_ws.is_empty() {
                        self.recover_environment_separator_prefix()
                    } else {
                        trailing_ws
                    };
                    row_separator_prefixes.push(separator_prefix);
                    self.pos += 1;
                    continue;
                }
                if self.consume_row_separator() {
                    break;
                }
                if self.starts_command("end") {
                    self.consume_end_environment(name)?;
                    rows.push(cells);
                    separator_leading.push(row_separator_prefixes);
                    return Ok(ParsedEnvironmentRows {
                        rows,
                        row_leading,
                        separator_leading,
                        end_leading: trailing_ws,
                    });
                }
                return Err(format!("expected &, \\\\, or \\end{{{name}}}"));
            }
            rows.push(cells);
            separator_leading.push(row_separator_prefixes);
        }
    }

    /// Parse array rows while preserving MathType''s probe-backed inter-row raw controls.
    fn parse_array_rows(&mut self, name: &str) -> Result<ParsedEnvironmentRows, String> {
        let mut rows = Vec::new();
        let mut row_leading = Vec::new();
        let mut separator_leading = Vec::new();
        loop {
            let leading_ws =
                normalize_environment_fallback_whitespace(&self.consume_raw_whitespace());
            if self.starts_command("end") {
                self.consume_end_environment(name)?;
                return Ok(ParsedEnvironmentRows {
                    rows,
                    row_leading,
                    separator_leading,
                    end_leading: String::new(),
                });
            }
            row_leading.push(self.consume_array_row_prefix(leading_ws));
            let mut cells = Vec::new();
            let mut row_separator_prefixes = Vec::new();
            loop {
                let cell = self.parse_sequence_until_environment_stop(name)?;
                cells.push(cell);
                let trailing_ws =
                    normalize_environment_fallback_whitespace(&self.consume_raw_whitespace());
                if self.peek() == Some('&') {
                    row_separator_prefixes.push(trailing_ws);
                    self.pos += 1;
                    continue;
                }
                if self.consume_row_separator() {
                    break;
                }
                if self.starts_command("end") {
                    self.consume_end_environment(name)?;
                    rows.push(cells);
                    separator_leading.push(row_separator_prefixes);
                    return Ok(ParsedEnvironmentRows {
                        rows,
                        row_leading,
                        separator_leading,
                        end_leading: String::new(),
                    });
                }
                return Err(format!("expected &, \\\\, or \\end{{{name}}}"));
            }
            rows.push(cells);
            separator_leading.push(row_separator_prefixes);
        }
    }

    /// Keep probe-known array rule commands so the writer can inject them between rows.
    fn consume_array_row_prefix(&mut self, mut leading_ws: String) -> String {
        let mut raw = String::new();
        loop {
            if self.starts_command("hline") {
                self.pos += 1 + "hline".len();
                leading_ws =
                    normalize_environment_fallback_whitespace(&self.consume_raw_whitespace());
                continue;
            }
            if self.starts_command("hdashline") {
                self.pos += 1 + "hdashline".len();
                raw.push_str(&leading_ws);
                raw.push_str("\\hdashline");
                let _ = self.consume_raw_whitespace();
                continue;
            }
            return raw;
        }
    }
    /// Parse a cell until an environment separator appears at the current nesting level.
    fn parse_sequence_until_environment_stop(&mut self, name: &str) -> Result<Expr, String> {
        let mut items = Vec::new();
        loop {
            let ws_start = self.pos;
            let _ = self.consume_raw_whitespace();
            if self.pos >= self.chars.len()
                || self.peek() == Some('&')
                || self.starts_command("end")
                || self.starts_row_separator()
            {
                self.pos = ws_start;
                break;
            }
            if let Some(infix) = self.consume_infix_command() {
                let left = Expr::Sequence(items);
                let right = self.parse_sequence_until_environment_stop(name)?;
                return Ok(infix_expr(infix, left, right));
            }
            let _ = name;
            let atom = self.parse_complete_atom()?;
            if expr_is_empty_sequence(&atom) {
                continue;
            }
            items.push(atom);
        }
        Ok(Expr::Sequence(items))
    }

    /// Match MathType TeX Input's known plain-text failure placeholder for unsupported commands.
    fn parse_mathtype_translation_failed_command(&mut self, label: &str) -> Result<Expr, String> {
        // MathType consumes the command argument but replaces the rendered result with a
        // plain-text failure notice instead of preserving the original TeX control word.
        let _ = self.parse_required_group(label)?;
        Ok(Expr::Text(MATHTYPE_TEXT_TRANSLATION_FAILED.to_string()))
    }

    /// Parse a braced one-column row stack such as \substack{a\\b}.
    fn parse_row_stack_group(&mut self, label: &str) -> Result<Vec<Vec<Expr>>, String> {
        self.expect('{')?;
        let mut rows = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some('}') {
                self.pos += 1;
                break;
            }
            let cell = self.parse_sequence_until_row_stack_stop()?;
            rows.push(vec![cell]);
            self.skip_ws();
            if self.consume_row_separator() {
                continue;
            }
            if self.peek() == Some('}') {
                self.pos += 1;
                break;
            }
            return Err(format!("expected \\\\ or closing brace for {label}"));
        }
        Ok(rows)
    }

    /// Parse one row-stack cell until a row separator or the closing brace.
    fn parse_sequence_until_row_stack_stop(&mut self) -> Result<Expr, String> {
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.pos >= self.chars.len()
                || self.peek() == Some('}')
                || self.starts_row_separator()
            {
                break;
            }
            if let Some(infix) = self.consume_infix_command() {
                let left = Expr::Sequence(items);
                let right = self.parse_sequence_until_row_stack_stop()?;
                return Ok(infix_expr(infix, left, right));
            }
            let atom = self.parse_complete_atom()?;
            if expr_is_empty_sequence(&atom) {
                continue;
            }
            items.push(atom);
        }
        Ok(Expr::Sequence(items))
    }

    /// Return true when the input is at a row separator command.
    fn starts_row_separator(&self) -> bool {
        self.peek() == Some('\\') && self.chars.get(self.pos + 1) == Some(&'\\')
    }

    /// Consume a row separator command if present.
    fn consume_row_separator(&mut self) -> bool {
        if self.starts_row_separator() {
            self.pos += 2;
            true
        } else {
            false
        }
    }

    /// Consume \end{name} for the currently parsed environment.
    fn consume_end_environment(&mut self, expected: &str) -> Result<(), String> {
        self.expect('\\')?;
        let start = self.pos;
        while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
            self.pos += 1;
        }
        let command: String = self.chars[start..self.pos].iter().collect();
        if command != "end" {
            return Err(format!("expected \\end{{{expected}}}, found \\{command}"));
        }
        let actual = self.parse_raw_group("environment end name")?;
        if actual != expected {
            return Err(format!(
                "expected \\end{{{expected}}}, found \\end{{{actual}}}"
            ));
        }
        Ok(())
    }

    /// Consume \right and return either a native delimiter char or one raw fallback token.
    fn parse_right_delimiter_spec(&mut self) -> Result<LeftRightDelimiter, String> {
        self.expect('\\')?;
        let start = self.pos;
        while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
            self.pos += 1;
        }
        let command: String = self.chars[start..self.pos].iter().collect();
        if command != "right" {
            return Err(format!("expected \\right, found \\{command}"));
        }
        self.parse_left_right_delimiter("right delimiter")
    }

    /// Parse one delimiter after \left or \right, preserving probe-known raw control words.
    fn parse_left_right_delimiter(&mut self, label: &str) -> Result<LeftRightDelimiter, String> {
        self.skip_ws();
        if self.peek() == Some('\\') {
            self.pos += 1;
            let start = self.pos;
            while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
                self.pos += 1;
            }
            if start != self.pos {
                let command: String = self.chars[start..self.pos].iter().collect();
                if raw_left_right_delimiter_command(&command) {
                    return Ok(LeftRightDelimiter::RawCommand(command));
                }
                return delimiter_command_char(&command)
                    .map(LeftRightDelimiter::Char)
                    .ok_or_else(|| format!("unsupported {label}: \\{command}"));
            }
            let ch = self.next().ok_or_else(|| format!("expected {label}"))?;
            return Ok(LeftRightDelimiter::Char(if ch == '|' {
                '\u{2016}'
            } else {
                ch
            }));
        }
        self.next()
            .map(LeftRightDelimiter::Char)
            .ok_or_else(|| format!("expected {label}"))
    }

    /// Parse one delimiter after \left or \right, including escaped braces.
    fn parse_delimiter_char(&mut self, label: &str) -> Result<char, String> {
        self.skip_ws();
        if self.peek() == Some('\\') {
            self.pos += 1;
            let start = self.pos;
            while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
                self.pos += 1;
            }
            if start != self.pos {
                let command: String = self.chars[start..self.pos].iter().collect();
                return delimiter_command_char(&command)
                    .ok_or_else(|| format!("unsupported {label}: \\{command}"));
            }
            let ch = self.next().ok_or_else(|| format!("expected {label}"))?;
            return Ok(if ch == '|' { '\u{2016}' } else { ch });
        }
        self.next().ok_or_else(|| format!("expected {label}"))
    }

    /// Consume \limits or \nolimits before scripts so limit-style functions keep the hint.
    fn consume_limits_modifier(&mut self) -> Option<LimitModifier> {
        for (command, modifier) in [
            ("limits", LimitModifier::Limits),
            ("nolimits", LimitModifier::NoLimits),
        ] {
            if self.starts_command(command) {
                self.pos += 1 + command.len();
                return Some(modifier);
            }
        }
        None
    }

    /// Consume an optional star used by commands such as \operatorname*.
    fn consume_optional_star(&mut self) -> bool {
        self.skip_ws();
        if self.peek() == Some('*') {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    /// Parse either a braced switch argument or the remaining local scope.
    fn parse_switch_content(&mut self, label: &str) -> Result<Expr, String> {
        self.skip_ws();
        if self.peek() == Some('{') {
            self.pos += 1;
            return self.parse_sequence(Some('}'));
        }
        let mut items = Vec::new();
        while !self.switch_content_stops() {
            items.push(self.parse_complete_atom()?);
            self.skip_ws();
        }
        if items.is_empty() {
            Err(format!("expected {label}"))
        } else {
            Ok(Expr::Sequence(items))
        }
    }

    /// Stop a switch command at the same local separators as ordinary sequences.
    fn switch_content_stops(&self) -> bool {
        self.pos >= self.chars.len()
            || self.starts_command("right")
            || matches!(self.peek(), Some('}' | '&'))
            || self.starts_row_separator()
    }

    /// Parse an optional bracketed group such as the index in \sqrt[n]{...}.
    fn parse_optional_bracket_group(&mut self) -> Result<Option<Expr>, String> {
        self.skip_ws();
        if self.peek() != Some('[') {
            return Ok(None);
        }
        self.pos += 1;
        Ok(Some(self.parse_sequence(Some(']'))?))
    }

    /// Keep HTML wrapper attributes visible because MathType emits them after the raw command name.
    fn parse_html_wrapper_content(&mut self, command: &str) -> Result<Expr, String> {
        let attribute = self.parse_visible_wrapper_group(&format!("{command} attribute"))?;
        let content = self.parse_visible_wrapper_group(&format!("{command} content"))?;
        Ok(raw_prefix_sequence(command, vec![attribute, content]))
    }

    /// Keep color-box arguments visible because MathType preserves them after the raw command name.
    fn parse_color_box_content(&mut self, has_frame: bool) -> Result<Expr, String> {
        let mut args = Vec::with_capacity(3);
        if has_frame {
            args.push(self.parse_visible_wrapper_group("fcolorbox frame color")?);
        }
        args.push(self.parse_visible_wrapper_group("colorbox background color")?);
        args.push(self.parse_visible_wrapper_group("colorbox content")?);
        Ok(raw_prefix_sequence(
            if has_frame { "fcolorbox" } else { "colorbox" },
            args,
        ))
    }

    /// Keep `\raisebox` as a raw prefix while exposing its consumed arguments visibly.
    fn parse_raisebox_content(&mut self) -> Result<Expr, String> {
        let lift = self.parse_raw_group("raisebox lift")?;
        let height = self.parse_optional_raw_bracket_group()?;
        let depth = self.parse_optional_raw_bracket_group()?;
        let content = self.parse_raw_group("raisebox content")?;
        let mut args = Vec::with_capacity(4);
        args.push(self.parse_visible_wrapper_text(&lift)?);
        if let Some(height) = height {
            args.push(self.parse_visible_wrapper_text(&format!("[{height}]"))?);
        }
        if let Some(depth) = depth {
            args.push(self.parse_visible_wrapper_text(&format!("[{depth}]"))?);
        }
        args.push(self.parse_visible_wrapper_text(&content)?);
        Ok(raw_prefix_sequence("raisebox", args))
    }

    /// Parse \hspace as a MathType-ignored layout hint when a length is present.
    fn parse_hspace_content(&mut self) -> Result<Expr, String> {
        self.consume_optional_star();
        let _ignored_width = self.parse_raw_group("hspace width")?;
        Ok(Expr::Sequence(Vec::new()))
    }

    /// Parse \cline as a MathType-ignored table-rule hint when a span is present.
    fn parse_cline_content(&mut self) -> Result<Expr, String> {
        let _ignored_span = self.parse_raw_group("cline span")?;
        Ok(Expr::Sequence(Vec::new()))
    }

    /// Parse a raw braced math argument so nested `$...$` delimiters are removed.
    fn parse_required_math_group(&mut self, label: &str) -> Result<Expr, String> {
        let content = self.parse_raw_group(label)?;
        Parser::new(&content).parse()
    }

    /// Parse one raw wrapper group as visible content so MathType-style hybrid wrappers can reuse it.
    fn parse_visible_wrapper_group(&mut self, label: &str) -> Result<Expr, String> {
        let content = self.parse_raw_group(label)?;
        self.parse_visible_wrapper_text(&content)
    }

    /// Parse one raw wrapper fragment so unsupported wrapper arguments stay visible in MathType order.
    fn parse_visible_wrapper_text(&self, content: &str) -> Result<Expr, String> {
        self.parse_macro_replacement(content)
    }

    /// Parse an optional raw bracket argument such as raisebox height/depth.
    fn parse_optional_raw_bracket_group(&mut self) -> Result<Option<String>, String> {
        self.skip_ws();
        if self.peek() != Some('[') {
            return Ok(None);
        }
        self.pos += 1;
        let start = self.pos;
        let mut depth = 1usize;
        while let Some(ch) = self.next() {
            match ch {
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        let end = self.pos - 1;
                        return Ok(Some(self.chars[start..end].iter().collect()));
                    }
                }
                _ => {}
            }
        }
        Err("unterminated optional bracket group".to_string())
    }

    /// Return true when the remaining input starts with a specific control word.
    fn starts_command(&self, expected: &str) -> bool {
        if self.peek() != Some('\\') {
            return false;
        }
        let mut index = self.pos + 1;
        for expected_char in expected.chars() {
            if self.chars.get(index) != Some(&expected_char) {
                return false;
            }
            index += 1;
        }
        !self
            .chars
            .get(index)
            .is_some_and(|ch| ch.is_ascii_alphabetic())
    }

    /// Return true when the remaining input is exactly \end{expected}.
    fn starts_end_environment(&self, expected: &str) -> bool {
        if !self.starts_command("end") {
            return false;
        }
        let mut index = self.pos + "\\end".len();
        if self.chars.get(index) != Some(&'{') {
            return false;
        }
        index += 1;
        for expected_char in expected.chars() {
            if self.chars.get(index) != Some(&expected_char) {
                return false;
            }
            index += 1;
        }
        self.chars.get(index) == Some(&'}')
    }

    /// Consume old TeX infix commands that map to existing native templates.
    fn consume_infix_command(&mut self) -> Option<InfixCommand> {
        let infix = if self.starts_command("over") {
            InfixCommand::Over
        } else if self.starts_command("above") {
            self.expect('\\').ok()?;
            while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
                self.pos += 1;
            }
            let thickness = self.parse_raw_group("above line thickness").ok()?;
            let thickness_expr = self.parse_visible_wrapper_text(&thickness).ok()?;
            return Some(InfixCommand::Above(Expr::Sequence(vec![
                Expr::RawTex(" \\above".to_string()),
                thickness_expr,
            ])));
        } else if self.starts_command("atop") {
            InfixCommand::Atop
        } else if self.starts_command("choose") {
            InfixCommand::Choose
        } else if self.starts_command("brace") {
            InfixCommand::Brace
        } else if self.starts_command("brack") {
            InfixCommand::Brack
        } else {
            return None;
        };
        self.expect('\\').ok()?;
        while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
            self.pos += 1;
        }
        Some(infix)
    }

    /// Preserve MathType's hybrid \genfrac path: raw command name plus visible arguments.
    fn parse_genfrac(&mut self) -> Result<Expr, String> {
        let left = self.parse_genfrac_delimiter("genfrac left delimiter")?;
        let right = self.parse_genfrac_delimiter("genfrac right delimiter")?;
        let thickness = self.parse_raw_group("genfrac line thickness")?;
        let style = self.parse_raw_group("genfrac style")?;
        let numerator = self.parse_required_group_or_atom("genfrac numerator")?;
        let denominator = self.parse_required_group_or_atom("genfrac denominator")?;
        let mut args = Vec::with_capacity(6);
        if let Some(left) = left {
            args.push(Expr::Char(left));
        }
        if let Some(right) = right {
            args.push(Expr::Char(right));
        }
        if !thickness.is_empty() {
            args.push(self.parse_visible_wrapper_text(&thickness)?);
        }
        if !style.is_empty() {
            args.push(self.parse_visible_wrapper_text(&style)?);
        }
        args.push(numerator);
        args.push(denominator);
        Ok(raw_prefix_sequence("genfrac", args))
    }

    /// Parse a \genfrac delimiter, where `{}` means no delimiter.
    fn parse_genfrac_delimiter(&mut self, label: &str) -> Result<Option<char>, String> {
        self.skip_ws();
        if self.peek() == Some('{') {
            let raw = self.parse_raw_group(label)?;
            return if raw.trim().is_empty() {
                Ok(None)
            } else {
                let mut parser = Parser::new(&raw);
                Ok(Some(parser.parse_delimiter_char(label)?))
            };
        }
        self.parse_delimiter_char(label).map(Some)
    }

    /// Parse a required fraction argument, accepting TeX's single-atom shorthand.
    fn parse_required_group_or_atom(&mut self, label: &str) -> Result<Expr, String> {
        self.skip_ws();
        if self.peek() == Some('{') {
            self.pos += 1;
            self.parse_sequence(Some('}'))
        } else if self.pos < self.chars.len() {
            self.parse_atom_with_scripts()
        } else {
            Err(format!("expected {label}"))
        }
    }

    /// Parse a required braced group and report which slot was missing.
    fn parse_required_group(&mut self, label: &str) -> Result<Expr, String> {
        self.skip_ws();
        if self.peek() != Some('{') {
            return Err(format!("expected braced {label}"));
        }
        self.pos += 1;
        self.parse_sequence(Some('}'))
    }

    /// Preserve prime-only superscript groups as raw source because MathType's
    /// TeX Input keeps forms like `^{'}`
    /// and `^{' '}` as fallback text.
    fn parse_raw_prime_script_group(&mut self) -> Result<Option<String>, String> {
        self.skip_ws();
        if self.peek() != Some('{') {
            return Ok(None);
        }
        let checkpoint = self.pos;
        let raw = self.parse_raw_group("prime superscript")?;
        if is_raw_prime_script_group(&raw) {
            Ok(Some(raw))
        } else {
            self.pos = checkpoint;
            Ok(None)
        }
    }

    /// Return raw text inside a simple braced group such as \operatorname{mean}.
    fn parse_raw_group(&mut self, label: &str) -> Result<String, String> {
        self.skip_ws();
        if self.peek() != Some('{') {
            return Err(format!("expected braced {label}"));
        }
        self.pos += 1;
        let start = self.pos;
        let mut depth = 1usize;
        while let Some(ch) = self.next() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        let end = self.pos - 1;
                        return Ok(self.chars[start..end].iter().collect());
                    }
                }
                _ => {}
            }
        }
        Err(format!("unterminated braced {label}"))
    }

    /// Consume a single required character.
    fn expect(&mut self, expected: char) -> Result<(), String> {
        match self.next() {
            Some(actual) if actual == expected => Ok(()),
            other => Err(format!("expected {expected:?}, found {other:?}")),
        }
    }

    /// Return the current character without consuming it.
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    /// Consume and return the current character.
    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += 1;
        Some(ch)
    }

    /// Ignore whitespace, matching MathType's treatment for simple TeX input.
    fn skip_ws(&mut self) {
        self.consume_ws();
    }

    /// Ignore whitespace and report whether at least one space was consumed.
    fn consume_ws(&mut self) -> bool {
        let start = self.pos;
        while self.peek().is_some_and(char::is_whitespace) {
            self.pos += 1;
        }
        self.pos != start
    }

    /// Consume one deferred space that belongs to the next raw fallback token.
    fn take_pending_raw_ws(&mut self) -> bool {
        let pending = self.pending_raw_ws;
        self.pending_raw_ws = false;
        pending
    }

    /// Consume raw source whitespace so fallback environments can reproduce
    /// MathType's row-leading trivia instead of one fixed separator prefix.
    fn consume_raw_whitespace(&mut self) -> String {
        let start = self.pos;
        while self.peek().is_some_and(char::is_whitespace) {
            self.pos += 1;
        }
        self.chars[start..self.pos].iter().collect()
    }

    /// Recover separator-adjacent source whitespace for fallback environments.
    ///
    /// This is a bug fix for rows like `a &= b`, where MathType preserves the
    /// literal space before `&` in raw fallback runs.
    fn recover_environment_separator_prefix(&self) -> String {
        let mut start = self.pos;
        while start > 0 && self.chars[start - 1].is_whitespace() {
            if matches!(self.chars[start - 1], '\r' | '\n') {
                break;
            }
            start -= 1;
        }
        self.chars[start..self.pos].iter().collect()
    }
}

/// Build the AST node for a TeX infix command after both sides are parsed.
fn infix_expr(command: InfixCommand, left: Expr, right: Expr) -> Expr {
    match command {
        InfixCommand::Over => Expr::Fraction(Box::new(left), Box::new(right)),
        InfixCommand::Above(command_expr) => raw_infix_expr(left, command_expr, right),
        InfixCommand::Atop => raw_infix_expr(left, Expr::RawTex(" \\atop".to_string()), right),
        InfixCommand::Choose => Expr::Pile {
            kind: PileKind::Parenthesized,
            upper: Box::new(left),
            lower: Box::new(right),
        },
        InfixCommand::Brace => raw_infix_expr(left, Expr::RawTex("\\brace".to_string()), right),
        InfixCommand::Brack => raw_infix_expr(left, Expr::RawTex("\\brack".to_string()), right),
    }
}

/// Preserve an old-TeX infix command between its visible left and right operands.
fn raw_infix_expr(left: Expr, command: Expr, right: Expr) -> Expr {
    Expr::Sequence(vec![left, command, right])
}

/// Append one superscript suffix that MathType preserves verbatim as raw TeX.
fn raw_superscript_suffix_expr(base: Expr, raw: &str) -> Expr {
    Expr::Sequence(vec![base, Expr::RawTex(format!("^{{{raw}}}"))])
}

/// Build native modulo text for TeX's parenthesized modulo operators.
fn modulo_parenthesized_expr(command: &str, argument: Expr) -> Expr {
    let content = if command == "pmod" {
        Expr::Sequence(vec![Expr::FunctionName("mod".to_string()), argument])
    } else {
        argument
    };
    Expr::Sequence(vec![
        Expr::Space(0x05),
        Expr::Delimited {
            left: '(',
            right: ')',
            content: Box::new(content),
        },
    ])
}

/// Return the visible relation character from a parsed relation atom.
fn relation_char(expr: &Expr) -> Option<char> {
    match expr {
        Expr::Char(ch) | Expr::CommandSymbol { ch, .. } => Some(*ch),
        Expr::Sequence(items) if items.len() == 1 => relation_char(&items[0]),
        _ => None,
    }
}

/// Preserve an unsupported prefix command while keeping its consumed operand visible.
fn raw_prefix_expr(command: &str, operand: Expr) -> Expr {
    Expr::Sequence(vec![Expr::RawTex(format!("\\{command}")), operand])
}

/// Preserve a raw command name while rendering each consumed argument as visible follow-up content.
fn raw_prefix_sequence(command: &str, args: Vec<Expr>) -> Expr {
    let mut items = Vec::with_capacity(args.len() + 1);
    items.push(Expr::RawTex(format!("\\{command}")));
    items.extend(args);
    Expr::Sequence(items)
}

/// Preserve a precomputed raw prefix while rendering each consumed argument as visible follow-up content.
fn raw_prefix_sequence_with_raw(raw_prefix: String, args: Vec<Expr>) -> Expr {
    let mut items = Vec::with_capacity(args.len() + 1);
    items.push(Expr::RawTex(raw_prefix));
    items.extend(args);
    Expr::Sequence(items)
}

/// Return true for unsupported environments that MathType stores as raw begin/end
/// commands plus visible interior tokens instead of one raw body blob.
fn unsupported_environment_uses_hybrid_begin_end(name: &str) -> bool {
    matches!(name, "CD")
}

/// Build one visible character sequence used by hybrid begin/end fallbacks.
fn visible_text_sequence(text: &str) -> Expr {
    Expr::Sequence(text.chars().map(Expr::Char).collect())
}
/// Flatten one visible wrapper group so raw-prefix hybrids keep MathType's item order.
fn push_visible_items(items: &mut Vec<Expr>, expr: Expr) {
    match expr {
        Expr::Sequence(seq) => items.extend(seq),
        other => items.push(other),
    }
}

/// Preserve one probe-known raw left-right control word or one native delimiter char.
fn push_left_right_delimiter(items: &mut Vec<Expr>, side: &str, delimiter: LeftRightDelimiter) {
    match delimiter {
        LeftRightDelimiter::Char(ch) => items.push(Expr::Char(ch)),
        LeftRightDelimiter::RawCommand(command) => {
            items.push(Expr::RawTex(format!("\\{side}\\{command}")));
        }
    }
}

/// Return true for control-word delimiters that MathType stores raw under left-right fences.
fn raw_left_right_delimiter_command(command: &str) -> bool {
    matches!(command, "lt" | "gt")
}

/// Split a raw-only macro replacement so MathType-style parameter markers keep their raw `#` bytes.
fn render_raw_only_macro_replacement(
    replacement: &str,
    parser: &Parser,
) -> Result<Vec<Expr>, String> {
    let chars = replacement.chars().collect::<Vec<_>>();
    let mut args = Vec::new();
    let mut start = 0usize;
    let mut index = 0usize;
    while index + 1 < chars.len() {
        if chars[index] == '#' && chars[index + 1] == '1' {
            if start < index {
                let chunk = chars[start..index].iter().collect::<String>();
                if !chunk.is_empty() {
                    args.push(parser.parse_visible_wrapper_text(&chunk)?);
                }
            }
            args.push(Expr::RawTex("#".to_string()));
            start = index + 1;
            index += 2;
            continue;
        }
        index += 1;
    }
    if start < chars.len() {
        let chunk = chars[start..].iter().collect::<String>();
        if !chunk.is_empty() {
            args.push(parser.parse_visible_wrapper_text(&chunk)?);
        }
    }
    Ok(args)
}

/// Return true when MathType keeps one following source-space inside the raw command run.
fn raw_command_preserves_trailing_space(command: &str) -> bool {
    matches!(command, "allowbreak")
}

/// Preserve spaces that MathType keeps inside raw-text fallback runs before unsupported commands.
fn with_leading_raw_space(expr: Expr, had_leading_ws: bool) -> Expr {
    if !had_leading_ws {
        return expr;
    }
    match expr {
        Expr::RawTex(text) => Expr::RawTex(format!(" {text}")),
        Expr::Sequence(mut items) => {
            if let Some(Expr::RawTex(text)) = items.first_mut() {
                text.insert(0, ' ');
            }
            Expr::Sequence(items)
        }
        other => other,
    }
}

/// MathType's raw fallback CHAR runs serialize line breaks as literal `n` bytes.
fn normalize_environment_fallback_whitespace(raw: &str) -> String {
    raw.chars()
        .map(|ch| match ch {
            '\r' | '\n' => 'n',
            other => other,
        })
        .collect()
}

/// Mirror MathType TeX Input's fallback behavior for direct literals MathType stores as raw fragments.
fn parse_literal_char(ch: char, had_leading_ws: bool) -> Result<Expr, String> {
    if let Some(fragments) = literal_raw_text_override(ch) {
        return Ok(literal_override_expr(fragments, had_leading_ws));
    }
    if ch.is_ascii() {
        return Ok(Expr::Char(ch));
    }
    let encoded = encode_mathtype_text(&ch.to_string())?;
    if encoded.iter().all(|byte| *byte == b'?') {
        let items = encoded.iter().map(|_| Expr::Char('?')).collect::<Vec<_>>();
        return Ok(match items.as_slice() {
            [item] => item.clone(),
            _ => Expr::Sequence(items),
        });
    }
    Ok(with_leading_raw_space(
        split_encoded_literal_bytes(&encoded),
        had_leading_ws,
    ))
}

/// Split one ANSI-encoded MathType literal into raw bytes plus visible ASCII fragments.
///
/// Some direct Unicode literals become mixed byte streams such as `0xA8 0x49`, where
/// MathType keeps the high byte raw but renders the trailing ASCII byte visibly.
fn split_encoded_literal_bytes(bytes: &[u8]) -> Expr {
    let mut items = Vec::new();
    let mut raw = Vec::new();
    for &byte in bytes {
        if byte.is_ascii() {
            if !raw.is_empty() {
                items.push(Expr::RawTex(raw.drain(..).map(char::from).collect()));
            }
            items.push(Expr::Char(byte as char));
        } else {
            raw.push(byte);
        }
    }
    if !raw.is_empty() {
        items.push(Expr::RawTex(raw.into_iter().map(char::from).collect()));
    }
    match items.as_slice() {
        [item] => item.clone(),
        _ => Expr::Sequence(items),
    }
}

/// Build one parser expression from generated direct-literal fallback fragments.
fn literal_override_expr(fragments: &[LiteralOverrideFragment], had_leading_ws: bool) -> Expr {
    let mut items = Vec::with_capacity(fragments.len());
    for fragment in fragments {
        match fragment {
            LiteralOverrideFragment::Raw(bytes) => {
                let mut raw = bytes.iter().copied().map(char::from).collect::<String>();
                if had_leading_ws && items.is_empty() {
                    raw.insert(0, ' ');
                }
                items.push(Expr::RawTex(raw));
            }
            LiteralOverrideFragment::Char(ch) => items.push(Expr::Char(*ch)),
        }
    }
    match items.len() {
        0 => Expr::Sequence(Vec::new()),
        1 => items
            .into_iter()
            .next()
            .expect("one literal override fragment"),
        _ => Expr::Sequence(items),
    }
}

/// Return true when a group can be stored as per-character MTEF embellishments.
fn is_simple_embellishment_run(raw: &str) -> bool {
    !raw.is_empty()
        && raw
            .chars()
            .all(|ch| !ch.is_whitespace() && !matches!(ch, '\\' | '{' | '}'))
}

/// Return true for parser-produced non-visible layout placeholders.
fn expr_is_empty_sequence(expr: &Expr) -> bool {
    matches!(expr, Expr::Sequence(items) if items.is_empty())
}


/// Drop one redundant wrapper sequence when a script group contains exactly one item.
fn collapse_single_sequence(expr: Expr) -> Expr {
    match expr {
        Expr::Sequence(mut items) if items.len() == 1 => items.remove(0),
        other => other,
    }
}
/// Return true when a braced superscript contains only apostrophes and spaces.
fn is_raw_prime_script_group(raw: &str) -> bool {
    !raw.is_empty()
        && raw.chars().any(|ch| ch == '\'')
        && raw.chars().all(|ch| matches!(ch, '\'' | ' '))
}

/// Return true for delimiter pairs that the writer can emit as scalable fences.
fn supported_delimiter_pair(left: char, right: char) -> bool {
    matches!(
        (left, right),
        ('(', ')')
            | ('[', ']')
            | ('{', '}')
            | ('|', '|')
            | ('\u{2016}', '\u{2016}')
            | ('\u{230a}', '\u{230b}')
            | ('\u{2308}', '\u{2309}')
            | ('\u{3008}', '\u{3009}')
            | ('<', '>')
    )
}

/// Map extensible-arrow commands that share MathType's x-arrow template.
fn xarrow_command_kind(command: &str) -> Option<XArrowKind> {
    match command {
        "xleftarrow" => Some(XArrowKind::Left),
        "xrightarrow" => Some(XArrowKind::Right),
        "xLeftarrow" => Some(XArrowKind::DoubleLeft),
        "xRightarrow" => Some(XArrowKind::DoubleRight),
        "xhookleftarrow" => Some(XArrowKind::HookLeft),
        "xhookrightarrow" => Some(XArrowKind::HookRight),
        "xtwoheadleftarrow" => Some(XArrowKind::TwoHeadLeft),
        "xtwoheadrightarrow" => Some(XArrowKind::TwoHeadRight),
        "xmapsto" => Some(XArrowKind::Mapsto),
        "xlongequal" => Some(XArrowKind::LongEqual),
        "xtofrom" => Some(XArrowKind::ToFrom),
        _ => None,
    }
}

/// Return true for x-arrow variants that MathType keeps as raw command text plus visible labels.
fn raw_hybrid_xarrow_command(command: &str) -> bool {
    matches!(
        command,
        "xLeftarrow"
            | "xRightarrow"
            | "xhookleftarrow"
            | "xhookrightarrow"
            | "xtwoheadleftarrow"
            | "xtwoheadrightarrow"
            | "xmapsto"
            | "xlongequal"
            | "xtofrom"
    )
}

/// Return the MathType logical-size style represented by a TeX style switch.
fn style_command_kind(command: &str) -> StyleKind {
    match command {
        "displaystyle" => StyleKind::Display,
        "textstyle" => StyleKind::Text,
        "scriptstyle" => StyleKind::Script,
        "scriptscriptstyle" => StyleKind::ScriptScript,
        _ => unreachable!("style_command_kind is only called for style switches"),
    }
}

/// Build a blackboard-bold single-letter alias such as \R or \Complex.
fn blackboard_letter(ch: char) -> Expr {
    Expr::Font {
        kind: FontKind::MathBb,
        content: Box::new(Expr::Char(ch)),
    }
}

/// Return true when a parsed expression already matches MathType's plain-text failure placeholder.
fn expr_is_mathtype_translation_failed(expr: &Expr) -> bool {
    match expr {
        Expr::Text(text) => text == MATHTYPE_TEXT_TRANSLATION_FAILED,
        // \\substack currently becomes a one-item sequence around the failure text,
        // and MathType collapses the surrounding scripted formula when that happens.
        Expr::Sequence(items) => matches!(items.as_slice(), [item] if expr_is_mathtype_translation_failed(item)),
        Expr::Style { content, .. } => expr_is_mathtype_translation_failed(content),
        _ => false,
    }
}

/// Preserve MathType's postfix script template shape by merging repeated scripts.
fn merge_script(
    base: Expr,
    sub: Option<Expr>,
    sup: Option<Expr>,
    limit_modifier: Option<LimitModifier>,
) -> Expr {
    if sub.as_ref().is_some_and(expr_is_mathtype_translation_failed)
        || sup.as_ref().is_some_and(expr_is_mathtype_translation_failed)
    {
        return Expr::Text(MATHTYPE_TEXT_TRANSLATION_FAILED.to_string());
    }
    match base {
        Expr::BigOp {
            kind,
            lower,
            upper,
            body,
        } => Expr::BigOp {
            kind,
            lower: sub.map(Box::new).or(lower),
            upper: sup.map(Box::new).or(upper),
            body,
        },
        Expr::Integral { kind } => Expr::IntegralOp {
            kind,
            lower: sub.map(Box::new),
            upper: sup.map(Box::new),
            body: None,
        },
        Expr::IntegralOp {
            kind,
            lower,
            upper,
            body,
        } => Expr::IntegralOp {
            kind,
            lower: sub.map(Box::new).or(lower),
            upper: sup.map(Box::new).or(upper),
            body,
        },
        Expr::FunctionName(name)
            if matches!(name.as_str(), "lim" | "sup")
                && limit_modifier == Some(LimitModifier::NoLimits) =>
        {
            Expr::Script {
                base: Box::new(Expr::FunctionName(name)),
                sub: sub.map(Box::new),
                sup: sup.map(Box::new),
            }
        }
        Expr::FunctionName(name) if matches!(name.as_str(), "lim" | "sup") => Expr::Limit {
            name,
            lower: sub.map(Box::new),
            upper: sup.map(Box::new),
        },
        Expr::Limit { name, lower, upper } => Expr::Limit {
            name,
            lower: sub.map(Box::new).or(lower),
            upper: sup.map(Box::new).or(upper),
        },
        Expr::Brace {
            kind,
            content,
            annotation,
        } if (kind == BraceKind::Under && sub.is_some())
            || (kind == BraceKind::Over && sup.is_some()) =>
        {
            Expr::Brace {
                kind,
                content,
                annotation: sub.or(sup).map(Box::new).or(annotation),
            }
        }
        Expr::Script {
            base,
            sub: old_sub,
            sup: old_sup,
        } => Expr::Script {
            base,
            sub: sub.map(Box::new).or(old_sub),
            sup: sup.map(Box::new).or(old_sup),
        },
        other => Expr::Script {
            base: Box::new(other),
            sub: sub.map(Box::new),
            sup: sup.map(Box::new),
        },
    }
}

/// Append a final environment row while keeping ownership straightforward.
fn rows_with_row(mut rows: Vec<Vec<Expr>>, row: Vec<Expr>) -> Vec<Vec<Expr>> {
    rows.push(row);
    rows
}

/// Map delimiter commands used after \left and \right to visible fence characters.
fn delimiter_command_char(command: &str) -> Option<char> {
    DELIMITER_COMMAND_CHARS
        .iter()
        .find(|entry| entry.command == command)
        .map(|entry| entry.ch)
        .or_else(|| command_to_char(command).filter(|ch| is_dynamic_delimiter_char(*ch)))
}

/// Return true for ordinary command aliases that can also act as delimiters.
fn is_dynamic_delimiter_char(ch: char) -> bool {
    matches!(
        ch,
        '(' | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | '|'
            | '\u{2016}'
            | '\u{230a}'
            | '\u{230b}'
            | '\u{2308}'
            | '\u{2309}'
            | '\u{3008}'
            | '\u{3009}'
            | '<'
            | '>'
    )
}

/// Map TeX spacing commands onto the fnSPACE bytes already verified by samples.
fn spacing_command_width(command: &str) -> Option<u8> {
    match command {
        "quad" => Some(0x05),
        "qquad" => Some(0x06),
        "medspace" => Some(0x02),
        "thickspace" => Some(0x04),
        "thinspace" | "space" | "nobreakspace" => Some(0x08),
        "negthinspace" | "negmedspace" | "negthickspace" => Some(0x01),
        _ => None,
    }
}

/// Map one-character TeX spacing escapes onto MathType's fnSPACE widths.
fn escaped_single_char_space(ch: char) -> Option<u8> {
    match ch {
        '!' => Some(0x01),
        ',' => Some(0x08),
        ':' | '>' => Some(0x02),
        ';' => Some(0x04),
        _ => None,
    }
}

/// Return true for delimiter-size hints that MathType drops while keeping the following fence.
fn ignored_delimiter_size_command(command: &str) -> bool {
    matches!(
        command,
        "big"
            | "Big"
            | "bigg"
            | "Bigg"
            | "bigl"
            | "Bigl"
            | "bigr"
            | "Bigr"
            | "bigm"
            | "Bigm"
            | "biggl"
            | "Biggl"
            | "biggr"
            | "Biggr"
            | "biggm"
            | "Biggm"
    )
}

/// Return escaped single-character commands that MathType stores as visible math glyphs.
fn escaped_single_char_math_char(ch: char) -> Option<char> {
    match ch {
        '#' | '%' | '&' | '_' => Some(ch),
        _ => None,
    }
}

/// Map no-argument LaTeX commands to the Unicode symbol MathType stores.
fn command_to_char(command: &str) -> Option<char> {
    TEX_COMMAND_CHARS
        .iter()
        .find(|entry| entry.command == command)
        .map(|entry| entry.ch)
}

/// Map no-argument LaTeX commands to short visible symbol sequences.
fn command_to_sequence(command: &str) -> Option<&'static [char]> {
    TEX_COMMAND_SEQUENCES
        .iter()
        .find(|entry| entry.command == command)
        .map(|entry| entry.chars)
}

/// Map no-argument LaTeX commands to text-style characters.
fn command_to_text(command: &str) -> Option<&'static str> {
    TEX_COMMAND_TEXTS
        .iter()
        .find(|entry| entry.command == command)
        .map(|entry| entry.text)
}

/// Map MathType big-symbol commands to the base glyph they write.
fn big_symbol_command_to_char(command: &str) -> Option<char> {
    BIG_SYMBOL_COMMAND_CHARS
        .iter()
        .find(|entry| entry.command == command)
        .map(|entry| entry.ch)
}

/// Map MathType tmSUMOP commands to the base glyph they write.
fn sum_operator_command_to_char(command: &str) -> Option<char> {
    SUM_OPERATOR_COMMAND_CHARS
        .iter()
        .find(|entry| entry.command == command)
        .map(|entry| entry.ch)
}

/// Return the logical character for source-command-aware symbols.
fn command_specific_to_char(command: &str) -> Option<char> {
    COMMAND_SPECIFIC_CHARS
        .iter()
        .find(|entry| entry.command == command)
        .map(|entry| entry.ch)
}










