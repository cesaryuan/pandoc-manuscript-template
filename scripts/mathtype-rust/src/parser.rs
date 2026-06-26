use crate::ast::*;
use crate::generated::char_tables::{
    BIG_SYMBOL_COMMAND_CHARS, COMMAND_SPECIFIC_CHARS, DELIMITER_COMMAND_CHARS,
    SUM_OPERATOR_COMMAND_CHARS, TEX_COMMAND_CHARS, TEX_COMMAND_SEQUENCES, TEX_COMMAND_TEXTS,
};
use std::collections::HashMap;

#[path = "parser/text_mode.rs"]
mod text_mode;

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
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MacroDefinition {
    params: usize,
    replacement: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InfixCommand {
    Over,
    Above,
    Atop,
    Choose,
    Brace,
    Brack,
}

impl Parser {
    /// Create a parser for the currently supported TeX math subset.
    pub(crate) fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            macros: HashMap::new(),
            expansion_depth: 0,
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
            self.skip_ws();
            if self.pos >= self.chars.len() || until.is_some_and(|end| self.peek() == Some(end)) {
                break;
            }
            if let Some(infix) = self.consume_infix_command() {
                let left = Expr::Sequence(items);
                let right = self.parse_sequence(until)?;
                return Ok(infix_expr(infix, left, right));
            }
            let atom = self.parse_complete_atom()?;
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
        self.skip_ws();
        match self.peek() {
            Some('{') => {
                self.pos += 1;
                self.parse_sequence(Some('}'))
            }
            Some('\\') => self.parse_command(),
            Some(ch) if ch != '}' => {
                self.pos += 1;
                Ok(Expr::Char(ch))
            }
            other => Err(format!("expected atom, found {other:?}")),
        }
    }

    /// Parse subscript/superscript arguments, accepting either groups or atoms.
    fn parse_script_arg(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if self.peek() == Some('{') {
            self.pos += 1;
            self.parse_sequence(Some('}'))
        } else {
            self.parse_atom()
        }
    }

    /// Parse supported LaTeX commands that map directly to MTEF templates.
    fn parse_command(&mut self) -> Result<Expr, String> {
        self.expect('\\')?;
        let start = self.pos;
        while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
            self.pos += 1;
        }
        let command: String = self.chars[start..self.pos].iter().collect();
        if command != "def" && command != "gdef" {
            if let Some(expanded) = self.expand_macro_command(&command)? {
                return Ok(expanded);
            }
        }
        match command.as_str() {
            "frac" | "dfrac" | "tfrac" => {
                let numerator = self.parse_required_group_or_atom("fraction numerator")?;
                let denominator = self.parse_required_group_or_atom("fraction denominator")?;
                Ok(Expr::Fraction(Box::new(numerator), Box::new(denominator)))
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
            "boxed" => Ok(Expr::Boxed(Box::new(
                self.parse_required_group("boxed content")?,
            ))),
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
            "oiint" => Ok(Expr::Integral {
                kind: IntegralKind::ContourDouble,
            }),
            "oiiint" => Ok(Expr::Integral {
                kind: IntegralKind::ContourTriple,
            }),
            "binom" | "dbinom" | "tbinom" => {
                let upper = self.parse_required_group("binomial upper")?;
                let lower = self.parse_required_group("binomial lower")?;
                Ok(Expr::Pile {
                    kind: PileKind::Parenthesized,
                    upper: Box::new(upper),
                    lower: Box::new(lower),
                })
            }
            "genfrac" => self.parse_genfrac(),
            "substack" => Ok(Expr::Matrix {
                kind: MatrixKind::Plain,
                rows: self.parse_row_stack_group("substack content")?,
            }),
            "begin" => {
                let name = self.parse_raw_group("environment name")?;
                self.parse_environment(&name)
            }
            "left" => {
                let left = self.parse_delimiter_char("left delimiter")?;
                let content = self.parse_sequence_until_right()?;
                let right = self.parse_right_delimiter()?;
                Ok(Expr::Delimited {
                    left,
                    right,
                    content: Box::new(content),
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
                let name = self.parse_raw_group("text color name")?;
                Ok(Expr::Color {
                    name,
                    content: Box::new(self.parse_required_group("textcolor content")?),
                })
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
            "operatorname" | "operatornamewithlimits" => {
                self.consume_optional_star();
                Ok(Expr::FunctionName(self.parse_raw_group("operator name")?))
            }
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
            | "varinjlim" | "varliminf" | "varlimsup" | "varprojlim" | "Pr" => {
                Ok(Expr::FunctionName(command))
            }
            "bmod" => Ok(Expr::FunctionName("mod".to_string())),
            "mod" => Ok(Expr::Sequence(vec![
                Expr::Space(0x05),
                Expr::FunctionName("mod".to_string()),
                Expr::Space(0x05),
            ])),
            "not" => self.parse_not_relation(),
            "pmod" | "pod" => {
                let argument = self.parse_required_group_or_atom("modulo argument")?;
                Ok(modulo_parenthesized_expr(command.as_str(), argument))
            }
            "ket" | "Ket" => Ok(Expr::Delimited {
                left: '|',
                right: '〉',
                content: Box::new(self.parse_required_group("ket content")?),
            }),
            "VERT" => Ok(Expr::Char('‖')),
            "bra" | "Bra" => Ok(bra_expr(self.parse_required_group("bra content")?)),
            "braket" | "Braket" => Ok(Expr::Delimited {
                left: '〈',
                right: '〉',
                content: Box::new(self.parse_required_group("braket content")?),
            }),
            "set" | "Set" => Ok(Expr::Delimited {
                left: '{',
                right: '}',
                content: Box::new(self.parse_required_group("set content")?),
            }),
            "rm" => Ok(self.parse_switch_content("rm content")?),
            "it" => Ok(self.parse_switch_content("it content")?),
            "mathrm" | "mathnormal" | "textnormal" | "textup" | "textmd" | "textrm" => {
                self.parse_required_group("roman content")
            }
            "mathit" | "textit" | "emph" => self.parse_required_group("italic content"),
            "bf" => Ok(Expr::Font {
                kind: FontKind::Bold,
                content: Box::new(self.parse_switch_content("bf content")?),
            }),
            "sf" => Ok(Expr::Font {
                kind: FontKind::MathSf,
                content: Box::new(self.parse_switch_content("sf content")?),
            }),
            "mathbf" | "textbf" | "bm" | "boldsymbol" | "bold" => Ok(Expr::Font {
                kind: FontKind::Bold,
                content: Box::new(self.parse_required_group("mathbf content")?),
            }),
            "pmb" => {
                self.skip_ws();
                if self.peek() == Some('{') {
                    Ok(Expr::Font {
                        kind: FontKind::Bold,
                        content: Box::new(self.parse_required_group("pmb content")?),
                    })
                } else {
                    Ok(Expr::RawTex("\\pmb".to_string()))
                }
            }
            "cal" => Ok(Expr::Font {
                kind: FontKind::MathCal,
                content: Box::new(self.parse_switch_content("cal content")?),
            }),
            "mathcal" => Ok(Expr::Font {
                kind: FontKind::MathCal,
                content: Box::new(self.parse_required_group("mathcal content")?),
            }),
            "mathsf" | "textsf" => Ok(Expr::Font {
                kind: FontKind::MathSf,
                content: Box::new(self.parse_required_group("mathsf content")?),
            }),
            "mathtt" | "texttt" => Ok(Expr::Font {
                kind: FontKind::MathTt,
                content: Box::new(self.parse_required_group("mathtt content")?),
            }),
            "tt" => Ok(Expr::Font {
                kind: FontKind::MathTt,
                content: Box::new(self.parse_switch_content("tt content")?),
            }),
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
            "underline" | "underbar" => Ok(Expr::BarTemplate {
                kind: BarTemplateKind::Under,
                content: Box::new(self.parse_required_group("underline content")?),
            }),
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
            "utilde" => self.parse_under_tilde_content(),
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
            "widecheck" => Ok(Expr::Accent {
                kind: AccentKind::Check,
                content: Box::new(self.parse_required_group("widecheck content")?),
            }),
            "vec" | "overrightarrow" | "Overrightarrow" => Ok(Expr::ArrowAccent {
                kind: ArrowAccentKind::Right,
                under: false,
                content: Box::new(self.parse_required_group("vector arrow content")?),
            }),
            "overleftarrow" => Ok(Expr::ArrowAccent {
                kind: ArrowAccentKind::Left,
                under: false,
                content: Box::new(self.parse_required_group("overleftarrow content")?),
            }),
            "underleftarrow" => Ok(Expr::ArrowAccent {
                kind: ArrowAccentKind::Left,
                under: true,
                content: Box::new(self.parse_required_group("underleftarrow content")?),
            }),
            "underrightarrow" => Ok(Expr::ArrowAccent {
                kind: ArrowAccentKind::Right,
                under: true,
                content: Box::new(self.parse_required_group("underrightarrow content")?),
            }),
            "overleftrightarrow" => Ok(Expr::ArrowAccent {
                kind: ArrowAccentKind::LeftRight,
                under: false,
                content: Box::new(self.parse_required_group("overleftrightarrow content")?),
            }),
            "underleftrightarrow" => Ok(Expr::ArrowAccent {
                kind: ArrowAccentKind::LeftRight,
                under: true,
                content: Box::new(self.parse_required_group("underleftrightarrow content")?),
            }),
            "overleftharpoon" => Ok(Expr::ArrowAccent {
                kind: ArrowAccentKind::LeftHarpoon,
                under: false,
                content: Box::new(self.parse_required_group("overleftharpoon content")?),
            }),
            "overrightharpoon" => Ok(Expr::ArrowAccent {
                kind: ArrowAccentKind::RightHarpoon,
                under: false,
                content: Box::new(self.parse_required_group("overrightharpoon content")?),
            }),
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
            "overbracket" => Ok(Expr::Bracket {
                kind: BracketKind::Over,
                content: Box::new(self.parse_required_group("overbracket content")?),
                annotation: None,
            }),
            "underbracket" => Ok(Expr::Bracket {
                kind: BracketKind::Under,
                content: Box::new(self.parse_required_group("underbracket content")?),
                annotation: None,
            }),
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
            "mathclap" | "mathllap" | "mathrlap" => self.parse_required_group("overlap content"),
            "vcenter" => self.parse_required_group("vcenter content"),
            "hspace" => self.parse_hspace_content(),
            "hline" | "hdashline" => Ok(Expr::Sequence(Vec::new())),
            "cline" => self.parse_cline_content(),
            "phase" => Ok(Expr::Sequence(vec![
                Expr::Char('∠'),
                self.parse_required_group("phase angle")?,
            ])),
            "raisebox" => self.parse_raisebox_content(),
            "smash" => {
                let _ignored_position = self.parse_optional_bracket_group()?;
                self.parse_required_group("smash content")
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
                command,
            }),
            _ if delimiter_command_char(&command).is_some() => {
                Ok(Expr::Char(delimiter_command_char(&command).unwrap()))
            }
            _ if spacing_command_width(&command).is_some() => {
                Ok(Expr::Space(spacing_command_width(&command).unwrap()))
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
                if ch == '!' {
                    return Ok(Expr::Space(0x01));
                }
                if ch == ',' {
                    return Ok(Expr::Space(0x08));
                }
                if ch.is_whitespace() {
                    return Ok(Expr::Space(0x08));
                }
                if ch == '|' {
                    return Ok(Expr::Char('‖'));
                }
                Ok(Expr::Char(ch))
            }
            _ => self.parse_unsupported_command(command),
        }
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
            name,
            MacroDefinition {
                params,
                replacement,
            },
        );
        Ok(Expr::Sequence(Vec::new()))
    }

    /// Expand a previously defined simple macro command, if one is in scope.
    fn expand_macro_command(&mut self, command: &str) -> Result<Option<Expr>, String> {
        let Some(definition) = self.macros.get(command).cloned() else {
            return Ok(None);
        };
        if self.expansion_depth >= 16 {
            return Ok(Some(Expr::RawTex(format!("\\{command}"))));
        }
        let expanded = match definition.params {
            0 => definition.replacement,
            1 => {
                let argument = match self.parse_raw_group("macro argument") {
                    Ok(argument) => argument,
                    Err(_) => return Ok(Some(Expr::RawTex(format!("\\{command}")))),
                };
                definition.replacement.replace("#1", &argument)
            }
            _ => return Ok(Some(Expr::RawTex(format!("\\{command}")))),
        };
        Ok(Some(self.parse_macro_replacement(&expanded)?))
    }

    /// Parse a replacement string with the current macro scope and depth limit.
    fn parse_macro_replacement(&self, replacement: &str) -> Result<Expr, String> {
        let mut parser = Parser {
            chars: replacement.chars().collect(),
            pos: 0,
            macros: self.macros.clone(),
            expansion_depth: self.expansion_depth + 1,
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

    /// Parse KaTeX-style hexadecimal Unicode escapes such as \char"263a.
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
        let code = u32::from_str_radix(&digits, 16)
            .map_err(|err| format!("invalid \\char hex code {digits}: {err}"))?;
        char::from_u32(code)
            .map(Expr::Char)
            .ok_or_else(|| format!("invalid Unicode scalar value in \\char\"{digits}"))
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
        let content = self.chars[start..self.pos]
            .iter()
            .map(|ch| Expr::Char(*ch))
            .collect();
        self.pos += 1;
        Ok(Expr::Font {
            kind: FontKind::MathTt,
            content: Box::new(Expr::Sequence(content)),
        })
    }

    /// Parse \not followed by a relation when Unicode has a stable negated form.
    fn parse_not_relation(&mut self) -> Result<Expr, String> {
        let relation = self.parse_required_group_or_atom("not relation")?;
        let Some(ch) = relation_char(&relation) else {
            return Ok(raw_prefix_expr("not", relation));
        };
        Ok(negated_relation_char(ch)
            .map(Expr::Char)
            .unwrap_or_else(|| raw_prefix_expr("not", relation)))
    }

    /// Preserve MathType TeX Input fallback for unsupported control words.
    fn parse_unsupported_command(&mut self, command: String) -> Result<Expr, String> {
        let raw = Expr::RawTex(format!("\\{command}"));
        self.skip_ws();
        if self.peek() == Some('{') {
            self.pos += 1;
            let argument = self.parse_sequence(Some('}'))?;
            Ok(Expr::Sequence(vec![raw, argument]))
        } else {
            Ok(raw)
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
        let mut atom = self.parse_atom()?;
        loop {
            self.skip_ws();
            if self.consume_limits_modifier() {
                continue;
            }
            match self.peek() {
                Some('_') => {
                    self.pos += 1;
                    let sub = self.parse_script_arg()?;
                    atom = merge_script(atom, Some(sub), None);
                }
                Some('^') => {
                    self.pos += 1;
                    let sup = self.parse_script_arg()?;
                    atom = merge_script(atom, None, Some(sup));
                }
                _ => break,
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
            "array" | "subarray" => {
                self.parse_raw_group("array column specifier")?;
                return Ok(Expr::Matrix {
                    kind: MatrixKind::Plain,
                    rows: self.parse_environment_rows(name)?,
                });
            }
            "matrix" | "smallmatrix" => {
                return Ok(Expr::Matrix {
                    kind: MatrixKind::Plain,
                    rows: self.parse_environment_rows(name)?,
                })
            }
            "pmatrix" => {
                return Ok(Expr::Matrix {
                    kind: MatrixKind::Parenthesized,
                    rows: self.parse_environment_rows(name)?,
                })
            }
            "bmatrix" => {
                return Ok(Expr::Matrix {
                    kind: MatrixKind::Bracketed,
                    rows: self.parse_environment_rows(name)?,
                })
            }
            "Bmatrix" => {
                return Ok(Expr::Matrix {
                    kind: MatrixKind::Braced,
                    rows: self.parse_environment_rows(name)?,
                })
            }
            "vmatrix" => {
                return Ok(Expr::Matrix {
                    kind: MatrixKind::Barred,
                    rows: self.parse_environment_rows(name)?,
                })
            }
            "Vmatrix" => {
                return Ok(Expr::Matrix {
                    kind: MatrixKind::DoubleBarred,
                    rows: self.parse_environment_rows(name)?,
                })
            }
            _ => return self.parse_unsupported_environment(name),
        };
        Ok(Expr::Environment {
            kind,
            rows: self.parse_environment_rows(name)?,
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

    /// Preserve an unsupported environment as one raw TeX fallback run.
    fn parse_unsupported_environment(&mut self, name: &str) -> Result<Expr, String> {
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

    /// Parse an environment into rows split by & and \\ separators.
    fn parse_environment_rows(&mut self, name: &str) -> Result<Vec<Vec<Expr>>, String> {
        let mut rows = Vec::new();
        loop {
            if self.starts_command("end") {
                self.consume_end_environment(name)?;
                break;
            }
            let mut cells = Vec::new();
            loop {
                let cell = self.parse_sequence_until_environment_stop(name)?;
                cells.push(cell);
                self.skip_ws();
                if self.peek() == Some('&') {
                    self.pos += 1;
                    continue;
                }
                if self.consume_row_separator() {
                    break;
                }
                if self.starts_command("end") {
                    self.consume_end_environment(name)?;
                    return Ok(rows_with_row(rows, cells));
                }
                return Err(format!("expected &, \\\\, or \\end{{{name}}}"));
            }
            rows.push(cells);
        }
        Ok(rows)
    }

    /// Parse a cell until an environment separator appears at the current nesting level.
    fn parse_sequence_until_environment_stop(&mut self, name: &str) -> Result<Expr, String> {
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.pos >= self.chars.len()
                || self.peek() == Some('&')
                || self.starts_command("end")
                || self.starts_row_separator()
            {
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

    /// Consume the \right command and return its visible delimiter.
    fn parse_right_delimiter(&mut self) -> Result<char, String> {
        self.expect('\\')?;
        let start = self.pos;
        while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
            self.pos += 1;
        }
        let command: String = self.chars[start..self.pos].iter().collect();
        if command != "right" {
            return Err(format!("expected \\right, found \\{command}"));
        }
        self.parse_delimiter_char("right delimiter")
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
            return Ok(if ch == '|' { '‖' } else { ch });
        }
        self.next().ok_or_else(|| format!("expected {label}"))
    }

    /// Consume \limits or \nolimits before scripts; layout is decided by writer templates.
    fn consume_limits_modifier(&mut self) -> bool {
        for command in ["limits", "nolimits"] {
            if self.starts_command(command) {
                self.pos += 1 + command.len();
                return true;
            }
        }
        false
    }

    /// Consume an optional star used by commands such as \operatorname*.
    fn consume_optional_star(&mut self) {
        self.skip_ws();
        if self.peek() == Some('*') {
            self.pos += 1;
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

    /// Strip KaTeX-only HTML attributes while keeping the math content visible.
    fn parse_html_wrapper_content(&mut self, command: &str) -> Result<Expr, String> {
        let _ignored_attribute = self.parse_raw_group(&format!("{command} attribute"))?;
        self.parse_required_group(&format!("{command} content"))
    }

    /// Keep color-box contents visible while ignoring unsupported box styling.
    fn parse_color_box_content(&mut self, has_frame: bool) -> Result<Expr, String> {
        if has_frame {
            let _ignored_frame_color = self.parse_raw_group("fcolorbox frame color")?;
        }
        let _ignored_background = self.parse_raw_group("colorbox background color")?;
        self.parse_required_math_group("colorbox content")
    }

    /// Keep raisebox contents visible while ignoring TeX-only box metrics.
    fn parse_raisebox_content(&mut self) -> Result<Expr, String> {
        let _ignored_lift = self.parse_raw_group("raisebox lift")?;
        let _ignored_height = self.parse_optional_raw_bracket_group()?;
        let _ignored_depth = self.parse_optional_raw_bracket_group()?;
        self.parse_required_math_group("raisebox content")
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
            InfixCommand::Above
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
        if matches!(infix, InfixCommand::Above) {
            self.parse_raw_group("above line thickness").ok()?;
        }
        Some(infix)
    }

    /// Parse \genfrac into existing fraction/pile and delimiter templates.
    fn parse_genfrac(&mut self) -> Result<Expr, String> {
        let left = self.parse_genfrac_delimiter("genfrac left delimiter")?;
        let right = self.parse_genfrac_delimiter("genfrac right delimiter")?;
        let thickness = self.parse_raw_group("genfrac line thickness")?;
        let _style = self.parse_raw_group("genfrac style")?;
        let numerator = self.parse_required_group_or_atom("genfrac numerator")?;
        let denominator = self.parse_required_group_or_atom("genfrac denominator")?;
        let body = if thickness.trim().is_empty() || thickness.trim() == "0pt" {
            Expr::Pile {
                kind: PileKind::Plain,
                upper: Box::new(numerator),
                lower: Box::new(denominator),
            }
        } else {
            Expr::Fraction(Box::new(numerator), Box::new(denominator))
        };
        Ok(match (left, right) {
            (None, None) => body,
            (Some(left), Some(right)) if supported_delimiter_pair(left, right) => Expr::Delimited {
                left,
                right,
                content: Box::new(body),
            },
            (Some(left), Some(right)) => {
                Expr::Sequence(vec![Expr::Char(left), body, Expr::Char(right)])
            }
            (Some(left), None) => Expr::Sequence(vec![Expr::Char(left), body]),
            (None, Some(right)) => Expr::Sequence(vec![body, Expr::Char(right)]),
        })
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
        while self.peek().is_some_and(char::is_whitespace) {
            self.pos += 1;
        }
    }
}

/// Build the AST node for a TeX infix command after both sides are parsed.
fn infix_expr(command: InfixCommand, left: Expr, right: Expr) -> Expr {
    match command {
        InfixCommand::Over | InfixCommand::Above => Expr::Fraction(Box::new(left), Box::new(right)),
        InfixCommand::Atop => Expr::Pile {
            kind: PileKind::Plain,
            upper: Box::new(left),
            lower: Box::new(right),
        },
        InfixCommand::Choose => Expr::Pile {
            kind: PileKind::Parenthesized,
            upper: Box::new(left),
            lower: Box::new(right),
        },
        InfixCommand::Brace => Expr::Pile {
            kind: PileKind::Braced,
            upper: Box::new(left),
            lower: Box::new(right),
        },
        InfixCommand::Brack => Expr::Pile {
            kind: PileKind::Bracketed,
            upper: Box::new(left),
            lower: Box::new(right),
        },
    }
}

/// Build native modulo text for TeX's parenthesized modulo operators.
fn modulo_parenthesized_expr(command: &str, argument: Expr) -> Expr {
    let mut items = vec![Expr::Space(0x05), Expr::Char('(')];
    if command == "pmod" {
        items.push(Expr::FunctionName("mod".to_string()));
        items.push(Expr::Space(0x05));
    }
    items.push(argument);
    items.push(Expr::Char(')'));
    Expr::Sequence(items)
}

/// Return the visible relation character from a parsed relation atom.
fn relation_char(expr: &Expr) -> Option<char> {
    match expr {
        Expr::Char(ch) | Expr::CommandSymbol { ch, .. } => Some(*ch),
        Expr::Sequence(items) if items.len() == 1 => relation_char(&items[0]),
        _ => None,
    }
}

/// Return one Unicode negated relation used by \not.
fn negated_relation_char(ch: char) -> Option<char> {
    match ch {
        '=' => Some('≠'),
        '<' => Some('≮'),
        '>' => Some('≯'),
        '∈' => Some('∉'),
        '∋' => Some('∌'),
        '∼' => Some('≁'),
        '≃' => Some('≄'),
        '≅' => Some('≇'),
        '≈' => Some('≉'),
        '≡' => Some('≢'),
        '≤' => Some('≰'),
        '≥' => Some('≱'),
        '≺' => Some('⊀'),
        '≻' => Some('⊁'),
        '⊂' => Some('⊄'),
        '⊃' => Some('⊅'),
        '⊆' => Some('⊈'),
        '⊇' => Some('⊉'),
        '⊑' => Some('⋢'),
        '⊒' => Some('⋣'),
        '⊢' => Some('⊬'),
        '⊨' => Some('⊭'),
        '⊩' => Some('⊮'),
        '⊫' => Some('⊯'),
        '←' => Some('↚'),
        '→' => Some('↛'),
        '↔' => Some('↮'),
        '⇐' => Some('⇍'),
        '⇒' => Some('⇏'),
        '⇔' => Some('⇎'),
        _ => None,
    }
}

/// Preserve an unsupported prefix command while keeping its consumed operand visible.
fn raw_prefix_expr(command: &str, operand: Expr) -> Expr {
    Expr::Sequence(vec![Expr::RawTex(format!("\\{command}")), operand])
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

/// Return true for delimiter pairs that the writer can emit as scalable fences.
fn supported_delimiter_pair(left: char, right: char) -> bool {
    matches!(
        (left, right),
        ('(', ')')
            | ('[', ']')
            | ('{', '}')
            | ('|', '|')
            | ('‖', '‖')
            | ('⌊', '⌋')
            | ('⌈', '⌉')
            | ('〈', '〉')
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

/// Build a native bra expression without guessing MathType's scalable bra template.
fn bra_expr(content: Expr) -> Expr {
    Expr::Sequence(vec![Expr::Char('〈'), content, Expr::Char('|')])
}

/// Build a blackboard-bold single-letter alias such as \R or \Complex.
fn blackboard_letter(ch: char) -> Expr {
    Expr::Font {
        kind: FontKind::MathBb,
        content: Box::new(Expr::Char(ch)),
    }
}

/// Preserve MathType's postfix script template shape by merging repeated scripts.
fn merge_script(base: Expr, sub: Option<Expr>, sup: Option<Expr>) -> Expr {
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
        Expr::Bracket {
            kind,
            content,
            annotation,
        } if (kind == BracketKind::Under && sub.is_some())
            || (kind == BracketKind::Over && sup.is_some()) =>
        {
            Expr::Bracket {
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
            | '‖'
            | '⌊'
            | '⌋'
            | '⌈'
            | '⌉'
            | '〈'
            | '〉'
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
