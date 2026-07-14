use crate::ast::*;
use crate::generated::char_tables::{
    BIG_SYMBOL_COMMAND_CHARS, COMMAND_SPECIFIC_CHARS, DELIMITER_COMMAND_CHARS,
    SUM_OPERATOR_COMMAND_CHARS, TEX_COMMAND_CHARS, TEX_COMMAND_SEQUENCES, TEX_COMMAND_TEXTS,
};
use crate::generated::color_tables::named_color_def;
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
    let_aliases: HashMap<String, String>,
    expansion_depth: usize,
    pending_raw_ws: String,
    active_unsupported_envs: Vec<String>,
    pending_closed_unsupported_envs: Vec<String>,
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
    row_annotations: Vec<EnvironmentRowAnnotation>,
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
    Command { command: String, ch: char },
    RawCommand(String),
}

impl Parser {
    /// Create a parser for the currently supported TeX math subset.
    pub(crate) fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            macros: HashMap::new(),
            let_aliases: HashMap::new(),
            expansion_depth: 0,
            pending_raw_ws: String::new(),
            active_unsupported_envs: Vec::new(),
            pending_closed_unsupported_envs: Vec::new(),
        }
    }

    /// Parse the full formula and reject trailing unsupported syntax.
    pub(crate) fn parse(mut self) -> Result<Expr, String> {
        let expr = merge_adjacent_raw_tex(self.parse_sequence(None)?);
        self.skip_ws();
        if self.pos != self.chars.len() {
            return Err(format!("unexpected character {:?}", self.peek()));
        }
        Ok(expr)
    }

    /// Parse the full formula without the final raw-fragment normalization step.
    ///
    /// This debug helper exists so local inspection tools can distinguish parser
    /// whitespace-loss bugs from later Sequence-level raw-run normalization bugs.
    #[allow(dead_code)]
    pub(crate) fn parse_unmerged(mut self) -> Result<Expr, String> {
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
            let leading_ws = self.consume_raw_whitespace();
            if self.pos >= self.chars.len() || until.is_some_and(|end| self.peek() == Some(end)) {
                break;
            }
            let infix_leading_ws = self.recover_environment_separator_prefix();
            if let Some(infix) = self.consume_infix_command() {
                if matches!(infix, InfixCommand::Over) && self.has_following_top_level_over(until) {
                    return self.parse_repeated_over_fallback(items, until);
                }
                let left = Expr::Sequence(items);
                let right = self.parse_sequence(until)?;
                return Ok(if infix_command_stays_native(&infix, until) {
                    infix_expr(infix, left, right)
                } else {
                    raw_infix_command_expr(infix, left, right, &infix_leading_ws)
                });
            }
            let atom = with_leading_raw_space(self.parse_complete_atom()?, &leading_ws);
            if expr_is_empty_sequence(&atom) {
                continue;
            }
            if expr_is_mathtype_translation_failed(&atom) {
                return Ok(atom);
            }
            if let Some(previous) = items.last_mut() {
                if expr_is_sticky_definition(previous) && !expr_is_sticky_definition(&atom) {
                    // Bug-fix: MathType keeps `\newcommand` / `\newenvironment`
                    // style definitions and one immediately following non-definition
                    // atom on the same fallback line instead of splitting them into
                    // two sibling Sequence records.
                    append_sticky_definition_followup(previous, atom);
                    continue;
                }
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
        let mut leading_ws = self.take_pending_raw_ws();
        leading_ws.push_str(&self.consume_raw_whitespace());
        match self.peek() {
            Some('{') => {
                let checkpoint = self.pos;
                self.pos += 1;
                match self.parse_sequence(Some('}')) {
                    Ok(group) => Ok(preserve_raw_group_braces(group)),
                    Err(_) => {
                        // Bug-fix: MathType keeps an unmatched `{` as raw text and continues
                        // instead of rejecting the whole formula.
                        self.pos = checkpoint + 1;
                        Ok(Expr::RawTex("{".to_string()))
                    }
                }
            }
            Some('\\') => self.parse_command(&leading_ws),
            Some('}') => {
                self.pos += 1;
                Ok(Expr::RawTex("}".to_string()))
            }
            Some('\'') => {
                self.pos += 1;
                // Bug-fix: a leading apostrophe in math mode stays on MathType's raw-text
                // path instead of becoming a visible prime embellishment without a base.
                Ok(with_leading_raw_space(
                    Expr::RawTex("'".to_string()),
                    &leading_ws,
                ))
            }
            Some(ch) if ch != '}' => {
                self.pos += 1;
                parse_literal_char(ch, &leading_ws)
            }
            other => Err(format!("expected atom, found {other:?}")),
        }
    }

    /// Consume one adjacent literal tail that MathType keeps on the text path.
    fn consume_adjacent_text_tail(&mut self) -> String {
        let start = self.pos;
        while self.peek().is_some_and(|ch| ch.is_ascii_alphanumeric()) {
            self.pos += 1;
        }
        self.chars[start..self.pos].iter().collect()
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
    fn parse_command(&mut self, leading_ws: &str) -> Result<Expr, String> {
        self.expect('\\')?;
        let start = self.pos;
        while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
            self.pos += 1;
        }
        let command: String = self.chars[start..self.pos].iter().collect();
        if command != "def" && command != "gdef" {
            if let Some(expanded) = self.expand_macro_command(&command)? {
                return Ok(with_leading_raw_space(expanded, leading_ws));
            }
        }
        if self
            .let_aliases
            .get(&command)
            .is_some_and(|target| target == "sqrt")
        {
            return self.parse_sqrt_let_alias(leading_ws);
        }
        let command = resolve_let_command_alias(&self.let_aliases, command);
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
            return self.parse_ignored_delimiter_size_command();
        }
        // MathType keeps some zero-argument aliases as raw source text even when scripts or
        // neighboring atoms follow, so preserve the command token before normal parsing.
        if should_force_raw_simple_command(&command) {
            return Ok(with_leading_raw_space(
                Expr::RawTex(format!("\\{command}")),
                leading_ws,
            ));
        }
        let expr = match command.as_str() {
            "frac" => self.parse_fraction_like_command(
                "frac",
                "fraction numerator",
                "fraction denominator",
                |numerator, denominator| Expr::Fraction(Box::new(numerator), Box::new(denominator)),
            ),
            "root" => self.parse_malformed_root_command(),
            "dfrac" => self.parse_fraction_like_command(
                "dfrac",
                "fraction numerator",
                "fraction denominator",
                |numerator, denominator| Expr::Style {
                    kind: StyleKind::Display,
                    content: Box::new(Expr::Fraction(Box::new(numerator), Box::new(denominator))),
                },
            ),
            "tfrac" => self.parse_fraction_like_command(
                "tfrac",
                "fraction numerator",
                "fraction denominator",
                |numerator, denominator| Expr::Style {
                    kind: StyleKind::Text,
                    content: Box::new(Expr::Fraction(Box::new(numerator), Box::new(denominator))),
                },
            ),
            "cfrac" => {
                let checkpoint = self.pos;
                if self.parse_optional_bracket_group()?.is_some() {
                    // Bug-fix: MathType rejects optional-alignment `\cfrac[l/r/c]{...}{...}`
                    // and collapses the whole command into its translation-failed placeholder.
                    let _ = self.parse_required_group_or_atom("continued fraction numerator");
                    let _ = self.parse_required_group_or_atom("continued fraction denominator");
                    Ok(Expr::Text(MATHTYPE_TEXT_TRANSLATION_FAILED.to_string()))
                } else {
                    self.pos = checkpoint;
                    self.parse_fraction_like_command(
                        "cfrac",
                        "continued fraction numerator",
                        "continued fraction denominator",
                        |numerator, denominator| {
                            Expr::Fraction(Box::new(numerator), Box::new(denominator))
                        },
                    )
                }
            }
            "sqrt" => {
                let checkpoint = self.pos;
                let index = match self.parse_optional_bracket_group() {
                    Ok(index) => index,
                    Err(_) => {
                        self.pos = checkpoint;
                        return Ok(with_leading_raw_space(
                            Expr::RawTex("\\sqrt".to_string()),
                            leading_ws,
                        ));
                    }
                };
                match self.parse_required_group_or_atom("square-root radicand") {
                    Ok(radicand) => {
                        if index.is_none() && malformed_closing_delimiter_raw(&radicand).is_some() {
                            // Bug-fix: malformed inputs such as `\sqrt}` stay on
                            // MathType's raw fallback path instead of opening a
                            // native root template with one visible `}` radicand.
                            return Ok(Expr::RawTex(format!(
                                "\\sqrt{}",
                                malformed_closing_delimiter_raw(&radicand)
                                    .expect("checked malformed sqrt closer")
                            )));
                        }
                        Ok(if let Some(index) = index {
                            Expr::NthRoot {
                                index: Box::new(index),
                                radicand: Box::new(radicand),
                            }
                        } else {
                            Expr::Sqrt(Box::new(radicand))
                        })
                    }
                    Err(_) => {
                        self.pos = checkpoint;
                        Ok(Expr::RawTex("\\sqrt".to_string()))
                    }
                }
            }
            "boxed" => self.parse_raw_prefix_group_command("boxed content", command.as_str()),
            "sum" => Ok(Expr::BigOp {
                kind: BigOpKind::Sum,
                lower: None,
                upper: None,
                body: None,
                placement: LimitPlacement::Limits,
            }),
            "prod" => Ok(Expr::BigOp {
                kind: BigOpKind::Product,
                lower: None,
                upper: None,
                body: None,
                placement: LimitPlacement::Limits,
            }),
            "coprod" => Ok(Expr::BigOp {
                kind: BigOpKind::Coproduct,
                lower: None,
                upper: None,
                body: None,
                placement: LimitPlacement::Limits,
            }),
            "bigcup" => Ok(Expr::BigOp {
                kind: BigOpKind::Union,
                lower: None,
                upper: None,
                body: None,
                placement: LimitPlacement::Limits,
            }),
            "bigcap" => Ok(Expr::BigOp {
                kind: BigOpKind::Intersection,
                lower: None,
                upper: None,
                body: None,
                placement: LimitPlacement::Limits,
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
            "iiiint" => Ok(quadruple_integral_expr()),
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
            "substack" => {
                let checkpoint = self.pos;
                if self.peek() == Some('{') {
                    Ok(Expr::Substack {
                        rows: self.parse_row_stack_group("substack content")?,
                    })
                } else {
                    match self.parse_switch_content("substack content") {
                        Ok(content) => Ok(content),
                        Err(_) => {
                            self.pos = checkpoint;
                            Ok(Expr::RawTex("\\substack".to_string()))
                        }
                    }
                }
            }
            "begin" => {
                let name = self.parse_raw_group("environment name")?;
                self.parse_environment(&name)
            }
            "end" => {
                let checkpoint = self.pos;
                match self.parse_raw_group("environment name") {
                    Ok(name) => Ok(Expr::RawTex(format!("\\end{{{name}}}"))),
                    Err(_) => {
                        self.pos = checkpoint;
                        Ok(Expr::RawTex("\\end".to_string()))
                    }
                }
            }
            "left" => self.parse_left_group(),
            "middle" => self.parse_middle_delimiter(),
            "!" => Ok(Expr::Space(0x01)),
            "," => Ok(Expr::Space(0x08)),
            "text" => {
                let checkpoint = self.pos;
                match self.parse_raw_group("text content") {
                    Ok(content) => {
                        let mut expr = text_mode::parse_content(&content);
                        if content.contains('$') {
                            let trailing = self.consume_adjacent_text_tail();
                            if !trailing.is_empty() {
                                // Bug-fix: after mixed text-mode `$...$` content,
                                // MathType may keep one adjacent literal tail on
                                // the same text run instead of reopening math
                                // variable glyphs immediately.
                                expr = append_visible_text_tail(expr, trailing);
                            }
                        }
                        Ok(expr)
                    }
                    Err(_) => {
                        self.pos = checkpoint;
                        match self.parse_required_group_or_atom("text content") {
                            // Bug-fix: MathType accepts bare-atom `\text A` shorthand and
                            // stores the visible atom under the same roman text font as `\text{A}`.
                            Ok(content) => Ok(Expr::Font {
                                kind: FontKind::RomanText,
                                content: Box::new(content),
                            }),
                            Err(_) => {
                                self.pos = checkpoint;
                                Ok(Expr::RawTex("\\text".to_string()))
                            }
                        }
                    }
                }
            }
            "mbox" | "hbox" => {
                let checkpoint = self.pos;
                match self.parse_raw_group("text box content") {
                    Ok(content) => {
                        if simple_text_box_group(&content) {
                            return Ok(text_mode::parse_content(&content));
                        }
                        if let Some(fallback) =
                            text_box_ref_like_fallback(command.as_str(), &content, self)?
                        {
                            return Ok(fallback);
                        }
                        let visible = self.parse_visible_wrapper_text(&content)?;
                        if expr_contains_raw_double_backslash(&visible) {
                            // Bug-fix: complex `\mbox{...}` content that already keeps
                            // a literal `\\` line-break fragment also keeps the outer
                            // brace shell on MathType's hybrid fallback path.
                            let mut items = vec![Expr::RawTex(format!("\\{command}{{"))];
                            push_visible_items(&mut items, visible);
                            items.push(Expr::RawTex("}".to_string()));
                            Ok(Expr::Sequence(items))
                        } else {
                            Ok(raw_prefix_expr(command.as_str(), visible))
                        }
                    }
                    Err(_) => {
                        self.pos = checkpoint;
                        Ok(Expr::RawTex(format!("\\{command}")))
                    }
                }
            }
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
            "ce" => self.parse_ce_content(),
            "label" => self.parse_ignored_group_command("label name", "label"),
            "notag" | "nonumber" => Ok(Expr::RawTex(format!("\\{command}"))),
            "ref" => self.parse_translation_failed_group_command("reference label", "ref"),
            "eqref" => {
                self.parse_translation_failed_group_command("equation reference label", "eqref")
            }
            "color" => {
                let checkpoint = self.pos;
                match self.parse_raw_group("color name") {
                    Ok(name) => {
                        let content = self.parse_switch_content("colored content")?;
                        if named_color_def(&name).is_some() {
                            Ok(Expr::Color {
                                name,
                                content: Box::new(content),
                            })
                        } else {
                            Ok(raw_prefix_sequence_with_raw(
                                format!("\\color{{{name}}}"),
                                vec![content],
                            ))
                        }
                    }
                    Err(_) => {
                        self.pos = checkpoint;
                        if self.parse_switch_content("malformed color content").is_ok() {
                            Ok(Expr::Text(MATHTYPE_TEXT_TRANSLATION_FAILED.to_string()))
                        } else {
                            self.pos = checkpoint;
                            Ok(Expr::RawTex("\\color".to_string()))
                        }
                    }
                }
            }
            "operatorname" => {
                let starred = self.consume_optional_star();
                let checkpoint = self.pos;
                if starred {
                    match self.parse_visible_wrapper_group("operator name") {
                        Ok(name) => Ok(Expr::Sequence(vec![
                            Expr::FunctionName("*".to_string()),
                            name,
                        ])),
                        Err(_) => {
                            self.pos = checkpoint;
                            Ok(Expr::RawTex("\\operatorname*".to_string()))
                        }
                    }
                } else {
                    match self.parse_raw_group("operator name") {
                        Ok(raw) => Ok(Expr::FunctionName(raw)),
                        Err(_) => {
                            self.pos = checkpoint;
                            match self.parse_required_group_or_atom("operator name") {
                                // Bug-fix: MathType accepts bare-atom `\operatorname A`
                                // shorthand and writes it as one function-style run whose
                                // first visible glyph is a leading space.
                                Ok(content) => {
                                    if let Some(text) = plain_operator_name_text(&content) {
                                        Ok(Expr::FunctionName(format!(" {text}")))
                                    } else {
                                        self.pos = checkpoint;
                                        Ok(Expr::RawTex("\\operatorname".to_string()))
                                    }
                                }
                                Err(_) => {
                                    self.pos = checkpoint;
                                    Ok(Expr::RawTex("\\operatorname".to_string()))
                                }
                            }
                        }
                    }
                }
            }
            "operatornamewithlimits" => self.parse_grouped_command_or_raw(
                "operatornamewithlimits",
                "operator name",
                |content| raw_prefix_expr("operatornamewithlimits", content),
            ),
            "mathop" => {
                let grouped = self.peek() == Some('{');
                let checkpoint = self.pos;
                match self.parse_required_group_or_atom("mathop content") {
                    Ok(content) => Ok(Expr::MathOp {
                        content: Box::new(content),
                        lower: None,
                        upper: None,
                        placement: LimitPlacement::Limits,
                        leading_space: !grouped,
                    }),
                    Err(_) => {
                        self.pos = checkpoint;
                        Ok(Expr::RawTex("\\mathop".to_string()))
                    }
                }
            }
            "matrixquantity"
            | "smallmatrixquantity"
            | "matrixdeterminant"
            | "mdet"
            | "smdet"
            | "mqty"
            | "smqty"
            | "pmqty"
            | "Pmqty"
            | "bmqty"
            | "vmqty"
            | "spmqty"
            | "sPmqty"
            | "sbmqty"
            | "svmqty"
            | "admat"
            | "dmat"
            | "imat"
            | "pmat" => self.parse_physics_matrix_command(command.as_str()),
            // Bug-fix: old-TeX matrix macros keep their wrapper command as raw TeX
            // while still exposing cell contents as visible LINE records.
            "array" | "matrix" | "pmatrix" => self.parse_old_tex_matrix_command(command.as_str()),
            // Bug-fix: old-TeX alignment wrappers keep the balanced brace shell
            // and raw top-level separators on MathType's fallback path.
            "eqalign" | "eqalignno" | "leqalignno" => {
                self.parse_old_tex_alignment_command(command.as_str())
            }
            "quantity" | "qty" => self.parse_physics_quantity_command(command.as_str()),
            "abs" | "norm" => self.parse_physics_raw_size_wrapper_command(
                command.as_str(),
                commands::PhysicsAutoBraceArity::One,
            ),
            "comm" | "acomm" | "commutator" | "anticommutator" | "poissonbracket" => self
                .parse_physics_raw_size_wrapper_command(
                    command.as_str(),
                    commands::PhysicsAutoBraceArity::Two,
                ),
            "cases" => self.parse_cases_command("cases"),
            "xmat" => self.parse_physics_multi_group_command("xmat", 3),
            "zmat" => self.parse_physics_multi_group_command("zmat", 2),
            "displaystyle" | "textstyle" | "scriptstyle" | "scriptscriptstyle" => Ok(Expr::Style {
                kind: style_command_kind(command.as_str()),
                content: Box::new(self.parse_switch_content("style switch content")?),
            }),
            "char" => self.parse_char_code(),
            "verb" => self.parse_verb_literal(),
            "newcommand" | "renewcommand" | "providecommand" => {
                self.parse_command_definition(&command)
            }
            "newenvironment" | "renewenvironment" => self.parse_environment_definition(&command),
            "def" | "gdef" => self.parse_macro_definition(&command),
            "let" => self.parse_let_command(),
            "futurelet" => Ok(Expr::RawTex(self.consume_raw_primitive(command))),
            "arg" | "argmax" | "argmin" | "arccos" | "arccot" | "arcctg" | "arccsc" | "arcsin"
            | "arcsec" | "arctan" | "arctg" | "ch" | "cos" | "cosec" | "cosh" | "cot" | "cotg"
            | "coth" | "csc" | "ctg" | "cth" | "deg" | "det" | "dim" | "exp" | "gcd" | "hom"
            | "injlim" | "ker" | "lg" | "lim" | "inf" | "liminf" | "limsup" | "ln" | "log"
            | "max" | "min" | "plim" | "projlim" | "sec" | "sh" | "sin" | "sinh" | "sup"
            | "tan" | "tanh" | "tg" | "th" | "Pr" => Ok(Expr::FunctionName(command)),
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
            "ket" | "Ket" => {
                self.parse_grouped_command_or_raw(command.as_str(), "ket content", |content| {
                    raw_prefix_expr(command.as_str(), content)
                })
            }
            "VERT" => Ok(Expr::RawTex("\\VERT".to_string())),
            "bra" | "Bra" => {
                self.parse_grouped_command_or_raw(command.as_str(), "bra content", |content| {
                    raw_prefix_expr(command.as_str(), content)
                })
            }
            "braket" | "Braket" => {
                self.parse_grouped_command_or_raw(command.as_str(), "braket content", |content| {
                    raw_prefix_expr(command.as_str(), content)
                })
            }
            "set" | "Set" => {
                self.parse_grouped_command_or_raw(command.as_str(), "set content", |content| {
                    raw_prefix_expr(command.as_str(), content)
                })
            }
            "rm" => Ok(Expr::Font {
                // MathType applies the legacy switch form like \mathrm, so the
                // whole switch span must render with fnTEXT rather than default
                // variable/number typefaces.
                kind: FontKind::RomanText,
                content: Box::new(self.parse_switch_content("rm content")?),
            }),
            "it" => Ok(self.parse_switch_content("it content")?),
            "mathrm" | "textrm" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "roman content",
                |content| Expr::Font {
                    kind: FontKind::RomanText,
                    content: Box::new(content),
                },
            ),
            "mathnormal" | "textnormal" | "textup" | "textmd" => self
                .parse_grouped_or_atom_command_or_raw(
                    command.as_str(),
                    "roman content",
                    |content| raw_prefix_expr(command.as_str(), content),
                ),
            "mathit" | "textit" | "emph" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "italic content",
                |content| content,
            ),
            "bf" => Ok(Expr::Font {
                kind: FontKind::Bold,
                content: Box::new(self.parse_switch_content("bf content")?),
            }),
            "sf" => Ok(raw_prefix_expr(
                "sf",
                self.parse_switch_content("sf content")?,
            )),
            "mathbf" | "textbf" | "boldsymbol" | "bold" => self
                .parse_grouped_or_atom_command_or_raw(
                    command.as_str(),
                    "mathbf content",
                    |content| Expr::Font {
                        kind: FontKind::Bold,
                        content: Box::new(content),
                    },
                ),
            "bm" => self.parse_grouped_command_or_raw("bm", "bm content", |content| {
                raw_prefix_expr("bm", content)
            }),
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
            "mathcal" => {
                let checkpoint = self.pos;
                match self.parse_required_group_or_atom("mathcal content") {
                    Ok(content) => Ok(Expr::Font {
                        kind: FontKind::MathCal,
                        content: Box::new(content),
                    }),
                    Err(_) => {
                        self.pos = checkpoint;
                        Ok(Expr::RawTex("\\mathcal".to_string()))
                    }
                }
            }
            "mathsf" => {
                self.parse_grouped_or_atom_command_or_raw("mathsf", "mathsf content", |content| {
                    Expr::Font {
                        kind: FontKind::MathSf,
                        content: Box::new(content),
                    }
                })
            }
            "textsf" => {
                self.parse_grouped_or_atom_command_or_raw("textsf", "textsf content", |content| {
                    raw_prefix_expr("textsf", content)
                })
            }
            "mathtt" => {
                self.parse_grouped_or_atom_command_or_raw("mathtt", "mathtt content", |content| {
                    raw_prefix_expr("mathtt", content)
                })
            }
            "texttt" => {
                self.parse_grouped_or_atom_command_or_raw("texttt", "texttt content", |content| {
                    Expr::Font {
                        kind: FontKind::TypewriterText,
                        content: Box::new(content),
                    }
                })
            }
            "tt" => Ok(raw_prefix_expr(
                "tt",
                self.parse_switch_content("tt content")?,
            )),
            "mathbb" | "Bbb" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "mathbb content",
                |content| Expr::Font {
                    kind: FontKind::MathBb,
                    content: Box::new(content),
                },
            ),
            "Complex" | "C" | "cnums" => Ok(blackboard_letter('C')),
            "Naturals" | "N" | "natnums" => Ok(blackboard_letter('N')),
            "Q" | "Rational" | "Rationals" => Ok(blackboard_letter('Q')),
            "R" | "Reals" | "reals" => Ok(blackboard_letter('R')),
            "Z" | "Integers" => Ok(blackboard_letter('Z')),
            "mathscr" => {
                self.parse_grouped_or_atom_command_or_raw("mathscr", "mathscr content", |content| {
                    Expr::Font {
                        kind: FontKind::MathScr,
                        content: Box::new(content),
                    }
                })
            }
            "mathfrak" => self.parse_grouped_or_atom_command_or_raw(
                "mathfrak",
                "mathfrak content",
                |content| Expr::Font {
                    kind: FontKind::MathFrak,
                    content: Box::new(content),
                },
            ),
            "bar" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "bar content",
                |content| Expr::Accent {
                    kind: AccentKind::Bar,
                    content: Box::new(content),
                },
            ),
            "overline" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "overline content",
                |content| Expr::BarTemplate {
                    kind: BarTemplateKind::Over,
                    content: Box::new(content),
                },
            ),
            "underline" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "underline content",
                |content| Expr::BarTemplate {
                    kind: BarTemplateKind::Under,
                    content: Box::new(content),
                },
            ),
            // MathType preserves the command token for \underbar instead of emitting
            // the native underbar template used by \underline.
            "underbar" => self.parse_raw_prefix_group_command("underbar content", "underbar"),
            "cancel" => self.parse_strike_command("cancel", "cancel content", StrikeKind::Up),
            "bcancel" => self.parse_strike_command("bcancel", "bcancel content", StrikeKind::Down),
            "xcancel" => self.parse_strike_command("xcancel", "xcancel content", StrikeKind::Both),
            "sout" => self.parse_strike_command("sout", "sout content", StrikeKind::Horizontal),
            "hat" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "hat content",
                |content| Expr::Accent {
                    kind: AccentKind::Hat,
                    content: Box::new(content),
                },
            ),
            "widehat" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "widehat content",
                |content| Expr::Accent {
                    kind: AccentKind::WideHat,
                    content: Box::new(content),
                },
            ),
            "breve" | "u" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "breve content",
                |content| Expr::Accent {
                    kind: AccentKind::Breve,
                    content: Box::new(content),
                },
            ),
            "dot" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "dot content",
                |content| Expr::Accent {
                    kind: AccentKind::Dot,
                    content: Box::new(content),
                },
            ),
            "ddot" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "ddot content",
                |content| Expr::Accent {
                    kind: AccentKind::Ddot,
                    content: Box::new(content),
                },
            ),
            "dddot" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "dddot content",
                |content| Expr::Accent {
                    kind: AccentKind::Dddot,
                    content: Box::new(content),
                },
            ),
            "ddddot" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "ddddot content",
                |content| Expr::Accent {
                    kind: AccentKind::Ddddot,
                    content: Box::new(content),
                },
            ),
            "tilde" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "tilde content",
                |content| Expr::Accent {
                    kind: AccentKind::Tilde,
                    content: Box::new(content),
                },
            ),
            // MathType preserves the command token for \utilde instead of
            // lowering it to native under-tilde embellishment records.
            "utilde" => self.parse_raw_prefix_group_command("utilde content", "utilde"),
            "acute" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "acute content",
                |content| Expr::Accent {
                    kind: AccentKind::Acute,
                    content: Box::new(content),
                },
            ),
            "grave" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "grave content",
                |content| Expr::Accent {
                    kind: AccentKind::Grave,
                    content: Box::new(content),
                },
            ),
            "check" | "v" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "check content",
                |content| Expr::Accent {
                    kind: AccentKind::Check,
                    content: Box::new(content),
                },
            ),
            "widecheck" => self.parse_raw_prefix_group_command("widecheck content", "widecheck"),
            "Overrightarrow" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "vector arrow content",
                |content| raw_prefix_expr(command.as_str(), content),
            ),
            "vec" | "overrightarrow" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "vector arrow content",
                |content| normalize_arrow_accent_expr(ArrowAccentKind::Right, false, content),
            ),
            "overleftarrow" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "overleftarrow content",
                |content| normalize_arrow_accent_expr(ArrowAccentKind::Left, false, content),
            ),
            "underleftarrow" | "underrightarrow" => self.parse_grouped_command_or_raw(
                command.as_str(),
                "under-arrow content",
                |content| raw_prefix_expr(command.as_str(), content),
            ),
            "overleftrightarrow" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "overleftrightarrow content",
                |content| normalize_arrow_accent_expr(ArrowAccentKind::LeftRight, false, content),
            ),
            "underleftrightarrow" => self.parse_grouped_command_or_raw(
                command.as_str(),
                "underleftrightarrow content",
                |content| raw_prefix_expr(command.as_str(), content),
            ),
            "overleftharpoon" | "overrightharpoon" => self.parse_grouped_command_or_raw(
                command.as_str(),
                "harpoon accent content",
                |content| raw_prefix_expr(command.as_str(), content),
            ),
            "overbrace" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "overbrace content",
                |content| Expr::Brace {
                    kind: BraceKind::Over,
                    content: Box::new(content),
                    annotation: None,
                },
            ),
            "underbrace" => self.parse_grouped_or_atom_command_or_raw(
                command.as_str(),
                "underbrace content",
                |content| Expr::Brace {
                    kind: BraceKind::Under,
                    content: Box::new(content),
                    annotation: None,
                },
            ),
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
                self.parse_grouped_command_or_raw("vcenter", "vcenter content", |content| {
                    raw_prefix_expr("vcenter", content)
                })
            }
            "hspace" => self.parse_hspace_content(),
            "hline" | "hdashline" => Ok(Expr::Sequence(Vec::new())),
            "cline" => self.parse_cline_content(),
            "phase" => {
                self.parse_grouped_command_or_raw(command.as_str(), "phase angle", |content| {
                    raw_prefix_expr(command.as_str(), content)
                })
            }
            _ if matches!(command.as_str(), "space" | "nobreakspace")
                && peek_non_whitespace_char(self).is_some_and(|ch| ch.is_ascii_alphanumeric()) =>
            {
                Ok(Expr::RawTex(format!("\\{command}")))
            }
            _ if spacing_command_width(&command).is_some() => {
                Ok(Expr::Space(spacing_command_width(&command).unwrap()))
            }
            _ if raw_hybrid_xarrow_command(&command) => {
                let checkpoint = self.pos;
                let under = self.parse_optional_bracket_group()?;
                match self.parse_required_group_or_atom("arrow label") {
                    Ok(label) => {
                        let mut args = vec![label];
                        if let Some(under) = under {
                            args.push(under);
                        }
                        Ok(raw_prefix_sequence(command.as_str(), args))
                    }
                    Err(_) => {
                        self.pos = checkpoint;
                        Ok(Expr::RawTex(format!("\\{command}")))
                    }
                }
            }
            _ if xarrow_command_kind(&command).is_some() => {
                let checkpoint = self.pos;
                let under = self.parse_optional_bracket_group()?;
                match self.parse_required_group_or_atom("arrow label") {
                    Ok(label) => Ok(Expr::XArrow {
                        kind: xarrow_command_kind(&command).unwrap(),
                        label: Box::new(label),
                        under: under.map(Box::new),
                    }),
                    Err(_) => {
                        self.pos = checkpoint;
                        Ok(Expr::RawTex(format!("\\{command}")))
                    }
                }
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
                if ch == '\\' {
                    if let Some(expr) = consume_visible_double_backslash_sequence(self) {
                        return Ok(with_leading_raw_space(expr, leading_ws));
                    }
                }
                if ch == '>' {
                    // Bug-fix: MathType drops `\>` entirely instead of mapping it to a spacing
                    // escape in math mode, even when visible content follows.
                    return Ok(with_leading_raw_space(
                        Expr::Sequence(Vec::new()),
                        leading_ws,
                    ));
                }
                if let Some(width) = escaped_single_char_space(ch) {
                    return Ok(Expr::Space(width));
                }
                if ch.is_whitespace() {
                    if !ch.is_ascii_whitespace() {
                        // Bug-fix: odd escapes such as `\NBSP` do not become a native
                        // control-space in MathType. It keeps the backslash raw and
                        // shows the unknown escaped character as a visible `?`.
                        return Ok(with_leading_raw_space(
                            Expr::Sequence(vec![Expr::RawTex("\\".to_string()), Expr::Char('?')]),
                            leading_ws,
                        ));
                    }
                    // MathType's escaped control-space uses the narrower fnSPACE 0xef04 slot,
                    // not the regular `\space` / `\nobreakspace` width.
                    return Ok(Expr::Space(0x04));
                }
                if ch == '&' {
                    // MathType TeX Input keeps `\&` on the raw-text path instead of converting it
                    // into a native punctuation CHAR record.
                    return Ok(with_leading_raw_space(
                        Expr::RawTex("\\&".to_string()),
                        leading_ws,
                    ));
                }
                if matches!(ch, '(' | ')' | '.') {
                    // Bug-fix: MathType stores `\(`, `\)`, and `\.` as one raw backslash
                    // followed by one visible punctuation glyph instead of a single escaped CHAR.
                    return Ok(with_leading_raw_space(
                        Expr::Sequence(vec![Expr::RawTex("\\".to_string()), Expr::Char(ch)]),
                        leading_ws,
                    ));
                }
                if matches!(ch, ']' | '<' | '>' | '/') {
                    // Bug-fix: MathType drops these escaped delimiter/correction shims instead of
                    // keeping a visible glyph or a raw fallback fragment in the final MTEF.
                    return Ok(with_leading_raw_space(
                        Expr::Sequence(Vec::new()),
                        leading_ws,
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
        Ok(with_leading_raw_space(expr, leading_ws))
    }
}

/// Preserve visible `\\foo` and `\\}`-style sequences using MathType's mixed raw/visible layout.
fn consume_visible_double_backslash_sequence(parser: &mut Parser) -> Option<Expr> {
    let start = parser.pos;
    if parser.peek().is_some_and(|next| next.is_ascii_alphabetic()) {
        while parser.peek().is_some_and(|next| next.is_ascii_alphabetic()) {
            parser.pos += 1;
        }
        let letters = parser.chars[start..parser.pos]
            .iter()
            .copied()
            .map(Expr::Char)
            .collect();
        Some(Expr::Sequence(vec![
            Expr::RawTex("\\\\".to_string()),
            Expr::Sequence(letters),
        ]))
    } else if parser
        .peek()
        .is_some_and(|next| matches!(next, '[' | '|') || next.is_ascii_whitespace())
    {
        while parser.peek().is_some_and(|next| next.is_ascii_whitespace()) {
            parser.pos += 1;
        }
        // Bug-fix: malformed visible line-break forms such as `\\[2ex]b`, `\\ A`,
        // and `\\|` keep only the raw `\\`; the following content stays on the
        // normal visible parse path instead of joining the raw run.
        Some(Expr::RawTex("\\\\".to_string()))
    } else if parser
        .peek()
        .is_some_and(|next| matches!(next, '{' | '}' | '<' | '>' | '/' | '.' | '(' | ')'))
    {
        parser.pos += 1;
        let suffix: String = parser.chars[start..parser.pos].iter().collect();
        Some(Expr::RawTex(format!("\\\\{suffix}")))
    } else {
        None
    }
}

/// Extract plain visible characters from a bare `\operatorname` atom shorthand.
fn plain_operator_name_text(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Char(ch) => Some(ch.to_string()),
        Expr::Sequence(items) if !items.is_empty() => {
            let mut text = String::new();
            for item in items {
                text.push_str(&plain_operator_name_text(item)?);
            }
            Some(text)
        }
        _ => None,
    }
}

/// Recognize the small `\mbox{A}`/`\hbox{x}` cases that MathType translates as plain text.
fn simple_text_box_group(content: &str) -> bool {
    !content.is_empty() && !content.contains(['\\', '$', '{', '}', '&', '^', '_'])
}

/// Return true when one wrapper payload keeps a literal `\\` raw fragment.
fn expr_contains_raw_double_backslash(expr: &Expr) -> bool {
    match expr {
        Expr::RawTex(raw) => raw.contains("\\\\"),
        Expr::DefaultColor(content)
        | Expr::Marked(content)
        | Expr::Style { content, .. }
        | Expr::Font { content, .. }
        | Expr::Color { content, .. } => expr_contains_raw_double_backslash(content),
        Expr::HybridLayout(parts) => parts.iter().any(|part| match part {
            crate::ast::HybridPart::Raw(raw) => raw.contains("\\\\"),
            crate::ast::HybridPart::Line(expr) => expr_contains_raw_double_backslash(expr),
        }),
        Expr::Sequence(items) => items.iter().any(expr_contains_raw_double_backslash),
        _ => false,
    }
}

/// Recover `\mbox{ \eqref{...} }` / `\hbox{ \ref{...} }` on MathType's mixed raw path.
///
/// Bug-fix: MathType keeps both wrapper command names as one raw run, drops the
/// surrounding braces, and only renders the reference label visibly.
fn text_box_ref_like_fallback(
    box_command: &str,
    content: &str,
    parser: &Parser,
) -> Result<Option<Expr>, String> {
    let leading_ws_len = content
        .find(|ch: char| !ch.is_whitespace())
        .unwrap_or(content.len());
    let leading_ws = &content[..leading_ws_len];
    let rest = &content[leading_ws_len..];

    for inner_command in ["eqref", "ref"] {
        let Some(after_command) = rest.strip_prefix(&format!("\\{inner_command}")) else {
            continue;
        };
        let Some((label, suffix)) = split_leading_braced_group(after_command) else {
            continue;
        };
        if !suffix.trim().is_empty() {
            continue;
        }
        let visible = parser.parse_visible_wrapper_text(label)?;
        if expr_is_mathtype_translation_failed(&visible) {
            continue;
        }
        let mut items = vec![Expr::RawTex(format!(
            "\\{box_command}{leading_ws}\\{inner_command}"
        ))];
        push_visible_items(&mut items, visible);
        return Ok(Some(collapse_single_sequence(Expr::Sequence(items))));
    }

    Ok(None)
}

/// Split one leading `{...}` group and return its raw content plus the remaining suffix.
fn split_leading_braced_group(text: &str) -> Option<(&str, &str)> {
    let mut chars = text.char_indices();
    let (start, first) = chars.next()?;
    if start != 0 || first != '{' {
        return None;
    }
    let mut depth = 1usize;
    for (index, ch) in chars {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let label = &text[1..index];
                    let suffix = &text[index + ch.len_utf8()..];
                    return Some((label, suffix));
                }
            }
            _ => {}
        }
    }
    None
}

/// Peek the next non-whitespace source character without consuming it.
fn peek_non_whitespace_char(parser: &Parser) -> Option<char> {
    let mut index = parser.pos;
    while parser.chars.get(index).is_some_and(|ch| ch.is_whitespace()) {
        index += 1;
    }
    parser.chars.get(index).copied()
}

/// Resolve one `\let` command alias before normal command dispatch.
fn resolve_let_command_alias(let_aliases: &HashMap<String, String>, command: String) -> String {
    let_aliases.get(&command).cloned().unwrap_or(command)
}

fn expr_is_sticky_definition(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Sequence(items)
            if items.first().is_some_and(|item| matches!(
                item,
                Expr::RawTex(raw)
                    if matches!(
                        raw.as_str(),
                        "\\newcommand"
                            | "\\renewcommand"
                            | "\\providecommand"
                    )
            ))
    )
}

/// Append one already parsed follow-up atom to the sticky definition Sequence.
fn append_sticky_definition_followup(target: &mut Expr, atom: Expr) {
    let Expr::Sequence(items) = target else {
        return;
    };
    items.push(normalize_definition_fallback_expr(atom));
}

/// Append one literal text tail to already-visible text-mode content.
fn append_visible_text_tail(expr: Expr, tail: String) -> Expr {
    match expr {
        Expr::Sequence(mut items) => {
            items.push(Expr::Text(tail));
            Expr::Sequence(items)
        }
        other => Expr::Sequence(vec![other, Expr::Text(tail)]),
    }
}

/// Downgrade bodyless big operators inside raw-definition fallback lines to plain glyphs.
fn normalize_definition_fallback_expr(expr: Expr) -> Expr {
    match expr {
        Expr::BigOp {
            kind,
            lower: None,
            upper: None,
            body: None,
            ..
        } => definition_bodyless_big_op_expr(kind),
        Expr::BigOp {
            kind,
            lower: None,
            upper: None,
            body: Some(body),
            ..
        } => Expr::FallbackBigOp {
            kind,
            body: Box::new(normalize_definition_fallback_expr(*body)),
        },
        Expr::Sequence(items) => Expr::Sequence(
            items
                .into_iter()
                .map(normalize_definition_fallback_expr)
                .collect(),
        ),
        other => other,
    }
}

/// Match MathType's fallback-line glyph choice for naked big operators in definitions.
fn definition_bodyless_big_op_expr(kind: BigOpKind) -> Expr {
    match kind {
        BigOpKind::Sum => Expr::Marked(Box::new(Expr::SumOperatorSymbol('\u{2211}'))),
        BigOpKind::Product => Expr::Marked(Box::new(Expr::BigSymbol('\u{220f}'))),
        BigOpKind::Coproduct => Expr::Marked(Box::new(Expr::BigSymbol('\u{2210}'))),
        BigOpKind::Union => Expr::Marked(Box::new(Expr::BigSymbol('\u{22c3}'))),
        BigOpKind::Intersection => Expr::Marked(Box::new(Expr::BigSymbol('\u{22c2}'))),
    }
}
