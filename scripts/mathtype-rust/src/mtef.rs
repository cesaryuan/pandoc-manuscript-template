use crate::ast::*;
use crate::generated::char_tables::ExplicitFont;
use crate::mathtype_ansi::{encode_mathtype_source, encode_mathtype_text};
use crate::typeface::{
    EXPLICIT_FONT_NEG_1, EXPLICIT_FONT_NEG_2, FN_FUNCTION, FN_MT_EXTRA, FN_NUMBER, FN_SPACE,
    FN_SYMBOL, FN_TEXT, FN_VARIABLE, FN_VECTOR,
};

#[path = "mtef/accents.rs"]
mod accents;
#[path = "mtef/delimiters.rs"]
mod delimiters;
#[path = "mtef/encoding.rs"]
mod encoding;
#[path = "mtef/environments.rs"]
mod environments;
#[path = "mtef/fixed_defs.rs"]
mod fixed_defs;
#[path = "mtef/predicates.rs"]
mod predicates;
#[path = "mtef/records.rs"]
mod records;
#[path = "mtef/scripts.rs"]
mod scripts;
#[path = "mtef/templates.rs"]
mod templates;

use accents::*;
use delimiters::*;
use environments::*;
use predicates::*;
use records::{
    color_black, color_default, write_expanding_glyph, write_null_line, write_size,
    write_table_char, write_table_char_with_embellishments, write_u16, write_unsigned,
};
use scripts::*;
use templates::*;

const EUCLID_MATH_ONE_DEFS: &[u8] = &[
    0x13, b'E', b'u', b'c', b'l', b'i', b'd', b'M', b'a', b't', b'h', b'1', 0x00, 0x11, 0x07, b'E',
    b'u', b'c', b'l', b'i', b'd', b' ', b'M', b'a', b't', b'h', b' ', b'O', b'n', b'e', 0x00, 0x08,
    0x06, 0x00,
];

const EUCLID_MATH_ONE_AFTER_TWO_DEFS: &[u8] = &[
    0x13, b'E', b'u', b'c', b'l', b'i', b'd', b'M', b'a', b't', b'h', b'1', 0x00, 0x11, 0x08, b'E',
    b'u', b'c', b'l', b'i', b'd', b' ', b'M', b'a', b't', b'h', b' ', b'O', b'n', b'e', 0x00, 0x08,
    0x07, 0x00,
];

const EUCLID_MATH_TWO_DEFS: &[u8] = &[
    0x13, b'E', b'u', b'c', b'l', b'i', b'd', b'M', b'a', b't', b'h', b'2', 0x00, 0x11, 0x07, b'E',
    b'u', b'c', b'l', b'i', b'd', b' ', b'M', b'a', b't', b'h', b' ', b'T', b'w', b'o', 0x00, 0x08,
    0x06, 0x00,
];

const EUCLID_MATH_TWO_AFTER_ONE_DEFS: &[u8] = &[
    0x13, b'E', b'u', b'c', b'l', b'i', b'd', b'M', b'a', b't', b'h', b'2', 0x00, 0x11, 0x08, b'E',
    b'u', b'c', b'l', b'i', b'd', b' ', b'M', b'a', b't', b'h', b' ', b'T', b'w', b'o', 0x00, 0x08,
    0x07, 0x00,
];

const EUCLID_FRAKTUR_DEFS: &[u8] = &[
    0x13, b'E', b'u', b'c', b'l', b'i', b'd', b'F', b'r', b'a', b'k', b't', b'u', b'r', 0x00, 0x11,
    0x07, b'E', b'u', b'c', b'l', b'i', b'd', b' ', b'F', b'r', b'a', b'k', b't', b'u', b'r', 0x00,
    0x08, 0x06, 0x00,
];

const EUCLID_FRAKTUR_AFTER_ONE_DEFS: &[u8] = &[
    0x13, b'E', b'u', b'c', b'l', b'i', b'd', b'F', b'r', b'a', b'k', b't', b'u', b'r', 0x00, 0x11,
    0x08, b'E', b'u', b'c', b'l', b'i', b'd', b' ', b'F', b'r', b'a', b'k', b't', b'u', b'r', 0x00,
    0x08, 0x07, 0x00,
];

const ARIAL_DEFS: &[u8] = &[
    0x11, 0x05, b'A', b'r', b'i', b'a', b'l', 0x00, 0x08, 0x06, 0x00,
];

const ARIAL_AFTER_ONE_DEFS: &[u8] = &[
    0x11, 0x06, b'A', b'r', b'i', b'a', b'l', 0x00, 0x08, 0x07, 0x00,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SizeState {
    Full,
    Sub,
    Sub2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ColorState {
    Default,
    Black,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WriteState {
    size: SizeState,
    color: ColorState,
}

struct MtefWriter {
    euclid_math_one_defined: bool,
    euclid_math_two_defined: bool,
    euclid_fraktur_defined: bool,
    sans_serif_defined: bool,
    euclid_math_one_typeface: u8,
    euclid_math_two_typeface: u8,
    euclid_fraktur_typeface: u8,
    sans_serif_typeface: u8,
    black_color_defined: bool,
    sans_serif_group_active: bool,
    typewriter_group_active: bool,
    big_symbol_line_marker_pending: bool,
    suppress_next_pile_color_default: bool,
    suppress_next_stackrel_color_default: bool,
    emit_top_fenced_matrix_color: bool,
    suppress_next_style_restore: bool,
    suppress_next_limit_restore: bool,
    emit_top_color_selector_one: bool,
    top_sequence_starts_default: bool,
    fallback_environment_active: bool,
    suppress_next_line_black: bool,
    line_starts_default: bool,
    parent_sequence_has_previous_sibling: bool,
}

impl MtefWriter {
    /// Emit the reusable black color definition once before selecting it.
    fn ensure_black_color_def(&mut self, out: &mut Vec<u8>) {
        if !self.black_color_defined {
            out.extend_from_slice(&[0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
            self.black_color_defined = true;
        }
    }

    /// Emit Euclid Math One once, at the position where MathType first needs it.
    fn ensure_euclid_math_one(&mut self, out: &mut Vec<u8>) {
        if !self.euclid_math_one_defined {
            if self.euclid_math_two_defined || self.euclid_fraktur_defined {
                out.extend_from_slice(EUCLID_MATH_ONE_AFTER_TWO_DEFS);
                self.euclid_math_one_typeface = EXPLICIT_FONT_NEG_2;
            } else {
                out.extend_from_slice(EUCLID_MATH_ONE_DEFS);
                self.euclid_math_one_typeface = EXPLICIT_FONT_NEG_1;
            }
            self.euclid_math_one_defined = true;
        }
    }

    /// Emit Euclid Math Two once for blackboard characters such as \mathbb{I}.
    fn ensure_euclid_math_two(&mut self, out: &mut Vec<u8>) {
        if !self.euclid_math_two_defined {
            if self.euclid_math_one_defined || self.euclid_fraktur_defined {
                out.extend_from_slice(EUCLID_MATH_TWO_AFTER_ONE_DEFS);
                self.euclid_math_two_typeface = EXPLICIT_FONT_NEG_2;
            } else {
                out.extend_from_slice(EUCLID_MATH_TWO_DEFS);
                self.euclid_math_two_typeface = EXPLICIT_FONT_NEG_1;
            }
            self.euclid_math_two_defined = true;
        }
    }

    /// Emit Euclid Fraktur once for MathType's native \mathfrak character table.
    fn ensure_euclid_fraktur(&mut self, out: &mut Vec<u8>) {
        if !self.euclid_fraktur_defined {
            if self.euclid_math_one_defined || self.euclid_math_two_defined {
                out.extend_from_slice(EUCLID_FRAKTUR_AFTER_ONE_DEFS);
                self.euclid_fraktur_typeface = EXPLICIT_FONT_NEG_2;
            } else {
                out.extend_from_slice(EUCLID_FRAKTUR_DEFS);
                self.euclid_fraktur_typeface = EXPLICIT_FONT_NEG_1;
            }
            self.euclid_fraktur_defined = true;
        }
    }

    /// Emit Arial once for native \mathsf output and remember which explicit slot it used.
    fn ensure_sans_serif(&mut self, out: &mut Vec<u8>) {
        if !self.sans_serif_defined {
            if self.euclid_math_one_defined
                || self.euclid_math_two_defined
                || self.euclid_fraktur_defined
            {
                out.extend_from_slice(ARIAL_AFTER_ONE_DEFS);
                self.sans_serif_typeface = EXPLICIT_FONT_NEG_2;
            } else {
                out.extend_from_slice(ARIAL_DEFS);
                self.sans_serif_typeface = EXPLICIT_FONT_NEG_1;
            }
            self.sans_serif_defined = true;
        }
    }
}

/// Build the MTEF stream, including MathType's TeX-source future record.
pub(crate) fn write_mtef(source_latex: &str, expr: &Expr) -> Result<Vec<u8>, String> {
    write_mtef_with_prefs(source_latex, expr, None)
}

/// Build the MTEF stream with optional per-equation MathType prefs.
pub(crate) fn write_mtef_with_prefs(
    source_latex: &str,
    expr: &Expr,
    prefs_file: Option<&std::path::Path>,
) -> Result<Vec<u8>, String> {
    let mut out = vec![0x05, 0x01, 0x00, 0x07, 0x08];
    out.extend_from_slice(b"DSMT7\0");
    // MathType switches to a shorter failure-form header when TeX Input collapses
    // the whole formula into "(Text translation failed)".
    if expr_is_translation_failed_placeholder(expr) {
        out.push(0x00);
    } else {
        out.push(0x01);
        out.push(0x66);
    }

    if !expr_is_translation_failed_placeholder(expr) {
        let mut source = b"TeX Input Language\0".to_vec();
        source.extend_from_slice(&encode_mathtype_source(source_latex)?);
        source.push(0x00);
        write_unsigned(source.len(), &mut out)?;
        out.extend_from_slice(&source);
    }

    if let Some(path) = prefs_file {
        out.extend_from_slice(&fixed_defs::fixed_defs_from_prefs_file(path)?);
    } else {
        out.extend_from_slice(fixed_defs::fixed_defs()?);
    }
    write_equation_body(expr, &mut out)?;
    Ok(out)
}

/// Write the outer Equation Native stream and keep its embedded MTEF length valid.
pub(crate) fn write_equation_native(mtef: &[u8]) -> Result<Vec<u8>, String> {
    let len = u32::try_from(mtef.len()).map_err(|_| "MTEF stream is too large".to_string())?;
    let mut out = vec![
        0x1c, 0x00, 0x00, 0x00, 0x02, 0x00, 0x3f, 0xc4, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x7c, 0xa2, 0x44, 0x17, 0x5d, 0xab, 0x97, 0x00, 0x0c, 0x00, 0xd8, 0x08,
    ];
    out[8..12].copy_from_slice(&len.to_le_bytes());
    out.extend_from_slice(mtef);
    Ok(out)
}

/// Write the top-level line, default black color, and equation terminators.
fn write_equation_body(expr: &Expr, out: &mut Vec<u8>) -> Result<(), String> {
    out.extend_from_slice(&[0x0a, 0x01, 0x00]);
    let mut writer = MtefWriter {
        euclid_math_one_defined: false,
        euclid_math_two_defined: false,
        euclid_fraktur_defined: false,
        sans_serif_defined: false,
        euclid_math_one_typeface: EXPLICIT_FONT_NEG_1,
        euclid_math_two_typeface: EXPLICIT_FONT_NEG_1,
        euclid_fraktur_typeface: EXPLICIT_FONT_NEG_1,
        sans_serif_typeface: EXPLICIT_FONT_NEG_1,
        black_color_defined: false,
        sans_serif_group_active: false,
        typewriter_group_active: false,
        big_symbol_line_marker_pending: false,
        suppress_next_pile_color_default: false,
        suppress_next_stackrel_color_default: false,
        emit_top_fenced_matrix_color: false,
        suppress_next_style_restore: false,
        suppress_next_limit_restore: false,
        emit_top_color_selector_one: false,
        top_sequence_starts_default: false,
        fallback_environment_active: false,
        suppress_next_line_black: false,
        line_starts_default: false,
        parent_sequence_has_previous_sibling: false,
    };
    if expr_is_only_spaces(expr) {
        write_only_spaces(expr, out)?;
        out.extend_from_slice(&[0x00, 0x00]);
        return Ok(());
    } else {
        let leading_space = top_leading_space_rest(expr);
        let starts_with_big_symbol_script = expr_starts_with_big_symbol_script_base(expr);
        let starts_with_euclid_math_one = expr_starts_with_euclid_math_one(expr);
        let starts_with_euclid_math_two = expr_starts_with_euclid_math_two(expr);
        let starts_with_euclid_fraktur = expr_starts_with_euclid_fraktur(expr);
        if starts_with_euclid_math_one {
            writer.ensure_euclid_math_one(out);
        }
        if starts_with_euclid_math_two {
            writer.ensure_euclid_math_two(out);
        }
        if starts_with_euclid_fraktur {
            writer.ensure_euclid_fraktur(out);
        }
        if starts_with_big_symbol_script {
            out.push(0x0d);
            writer.big_symbol_line_marker_pending = true;
        } else if expr_starts_with_standalone_integral(expr)
            || expr_starts_with_standalone_big_glyph(expr)
        {
            out.push(0x0d);
        }
        let starts_with_bodyless_big_op_script = expr_starts_with_bodyless_big_op_script(expr);
        let starts_with_top_pile_template = expr_starts_with_top_pile_template(expr);
        let starts_with_top_fenced_matrix = expr_starts_with_top_fenced_matrix(expr);
        let starts_with_top_style = expr_starts_with_top_style(expr);
        let starts_with_top_color = expr_starts_with_top_color(expr);
        if let Some((width, rest)) = leading_space {
            write_space_without_color(width, out);
            if rest.is_empty() {
                out.extend_from_slice(&[0x00, 0x00]);
                return Ok(());
            }
            writer.ensure_black_color_def(out);
            color_black(out);
            let rest_expr = Expr::Sequence(rest);
            write_expr(&rest_expr, out, SizeState::Full, &mut writer)?;
            out.extend_from_slice(&[0x00, 0x00]);
            return Ok(());
        }
        if !expr_starts_with_top_matrix(expr)
            && !expr_starts_with_self_opening(expr)
            && !starts_with_bodyless_big_op_script
            && !expr_starts_with_slotless_big_op(expr)
            && !starts_with_top_pile_template
            && !starts_with_top_style
            && !expr_sets_own_color(expr)
            && !expr_is_translation_failed_placeholder(expr)
        {
            writer.ensure_black_color_def(out);
            color_black(out);
        }
        writer.suppress_next_pile_color_default = starts_with_top_pile_template;
        writer.suppress_next_stackrel_color_default = expr_starts_with_stackrel(expr);
        writer.emit_top_fenced_matrix_color = starts_with_top_fenced_matrix;
        writer.suppress_next_style_restore = starts_with_top_style;
        writer.suppress_next_limit_restore = expr_is_standalone_limit(expr);
        writer.emit_top_color_selector_one = starts_with_top_color;
        writer.top_sequence_starts_default =
            expr_starts_with_sum_operator_script_base(expr) || starts_with_bodyless_big_op_script;
    }
    write_expr(expr, out, SizeState::Full, &mut writer)?;
    out.extend_from_slice(&[0x00, 0x00]);
    Ok(())
}

/// Write an expression in MathType's record order for the supported subset.
fn write_expr(
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let next_state = match expr {
        Expr::Sequence(items) => {
            let starts_with_explicit_accent = expr_starts_with_explicit_accent_template(expr);
            let start_default = writer.top_sequence_starts_default
                || (writer.line_starts_default
                    && items
                        .first()
                        .is_some_and(expr_starts_with_line_layout_object));
            if start_default {
                writer.top_sequence_starts_default = false;
            }
            let mut state = WriteState {
                size: current_size,
                color: if start_default {
                    ColorState::Default
                } else {
                    ColorState::Black
                },
            };
            for (index, item) in items.iter().enumerate() {
                if state.size != current_size && !matches!(item, Expr::Integral { .. }) {
                    if expr_starts_with_euclid_math_one(item) {
                        writer.ensure_euclid_math_one(out);
                    } else if expr_starts_with_euclid_math_two(item) {
                        writer.ensure_euclid_math_two(out);
                    }
                    write_size(current_size, out);
                    state.size = current_size;
                }
                if let Expr::Space(width) = item {
                    if state.color == ColorState::Default {
                        write_space_without_color(*width, out);
                        state = WriteState {
                            size: state.size,
                            color: ColorState::Default,
                        };
                        continue;
                    }
                }
                let needs_black_selector = !(state.color == ColorState::Black
                    || (index == 0 && start_default)
                    || expr_starts_with_line_font_def(item)
                    || expr_starts_with_empty_base_superscript(item)
                    || expr_sets_own_color(item));
                if needs_black_selector {
                    if expr_starts_with_euclid_math_one(item) {
                        writer.ensure_euclid_math_one(out);
                    } else if expr_starts_with_euclid_math_two(item) {
                        writer.ensure_euclid_math_two(out);
                    } else if expr_starts_with_euclid_fraktur(item) {
                        writer.ensure_euclid_fraktur(out);
                    }
                    writer.ensure_black_color_def(out);
                    color_black(out);
                    state.color = ColorState::Black;
                }
                if state.color == ColorState::Black
                    && expr_starts_with_sum_operator_script_base(item)
                {
                    color_default(out);
                    state.color = ColorState::Default;
                }
                if state.color == ColorState::Black && expr_starts_with_bodyless_big_op_script(item)
                {
                    color_default(out);
                    state.color = ColorState::Default;
                }
                if index > 0 && state.color == ColorState::Black && expr_starts_with_limit(item) {
                    color_default(out);
                    state.color = ColorState::Default;
                }
                if index > 0 && state.color == ColorState::Black && expr_starts_with_raw_tex(item) {
                    color_default(out);
                    state.color = ColorState::Default;
                }
                if index > 0
                    && state.color == ColorState::Black
                    && expr_starts_with_binom_pile(item)
                {
                    color_default(out);
                    state.color = ColorState::Default;
                }
                if index > 0
                    && state.color == ColorState::Black
                    && expr_starts_with_top_matrix(item)
                    && items[index - 1].contains_raw_tex()
                {
                    color_default(out);
                    state.color = ColorState::Default;
                }
                if index > 0
                    && state.color == ColorState::Black
                    && expr_starts_with_environment_fallback(item)
                {
                    color_default(out);
                    state.color = ColorState::Default;
                }
                if state.color == ColorState::Black
                    && expr_starts_with_explicit_accent_template(item)
                    && !(index == 0 && starts_with_explicit_accent)
                {
                    color_default(out);
                    state.color = ColorState::Default;
                }
                if index == 0 && start_default && expr_starts_with_top_style(item) {
                    writer.top_sequence_starts_default = true;
                }
                let previous_parent_sequence_has_previous_sibling =
                    writer.parent_sequence_has_previous_sibling;
                writer.parent_sequence_has_previous_sibling = index > 0;
                state = write_expr(item, out, state.size, writer)?;
                writer.parent_sequence_has_previous_sibling =
                    previous_parent_sequence_has_previous_sibling;
            }
            state
        }
        Expr::Char(ch) => {
            write_char(*ch, out, writer)?;
            WriteState {
                size: current_size,
                color: ColorState::Black,
            }
        }
        Expr::MarkedChar(ch) => {
            // Some MathType delimiter-size hints collapse into a plain visible glyph that still
            // keeps the standalone line marker byte in front of the CHAR record.
            out.push(0x0d);
            write_char(*ch, out, writer)?;
            WriteState {
                size: current_size,
                color: ColorState::Black,
            }
        }
        Expr::NotRelation(content) => {
            write_not_relation(content, out, writer)?;
            WriteState {
                size: current_size,
                color: ColorState::Black,
            }
        }
        Expr::CommandSymbol { command, ch } => {
            write_command_symbol(command, *ch, out, writer)?;
            WriteState {
                size: current_size,
                color: ColorState::Black,
            }
        }
        Expr::BigSymbol(ch) => {
            write_big_symbol_char(*ch, out, writer)?;
            WriteState {
                size: current_size,
                color: ColorState::Black,
            }
        }
        Expr::SumOperatorSymbol(ch) => {
            write_sum_operator_glyph(*ch, out, writer)?;
            WriteState {
                size: current_size,
                color: ColorState::Black,
            }
        }
        Expr::RawTex(text) => {
            write_raw_tex_text(text, out)?;
            WriteState {
                size: current_size,
                color: ColorState::Default,
            }
        }
        Expr::Space(width) => {
            write_space(*width, out);
            WriteState {
                size: current_size,
                color: ColorState::Default,
            }
        }
        Expr::FunctionName(name) => {
            write_function_name(name, out, writer)?;
            WriteState {
                size: current_size,
                color: ColorState::Black,
            }
        }
        Expr::Text(text) => {
            write_text(text, out)?;
            WriteState {
                size: current_size,
                color: ColorState::Black,
            }
        }
        Expr::Color { name, content } => {
            write_color_expr(name, content, out, current_size, writer)?
        }
        Expr::Style { kind, content } => {
            write_style_expr(*kind, content, out, current_size, writer)?
        }
        Expr::Font { kind, content } => write_font_expr(*kind, content, out, current_size, writer)?,
        Expr::Accent { kind, content } => {
            write_accent_expr(*kind, content, out, current_size, writer)?
        }
        Expr::ArrowAccent {
            kind,
            under,
            content,
        } => write_arrow_accent_template(*kind, *under, content, out, current_size, writer)?,
        Expr::BarTemplate { kind, content } => {
            write_bar_template(*kind, content, out, current_size, writer)?
        }
        Expr::Strike { kind, content } => {
            write_strike_template(*kind, content, out, current_size, writer)?
        }
        Expr::Fraction(numerator, denominator) => {
            write_fraction(numerator, denominator, out, current_size, writer)?
        }
        Expr::Sqrt(radicand) => write_sqrt(radicand, out, current_size, writer)?,
        Expr::NthRoot { index, radicand } => {
            write_nth_root(index, radicand, out, current_size, writer)?
        }
        Expr::BigOp {
            kind,
            lower,
            upper,
            body,
        } => write_big_op(
            *kind,
            body.as_deref(),
            lower.as_deref(),
            upper.as_deref(),
            out,
            current_size,
            writer,
        )?,
        Expr::Limit { name, lower, upper } => write_limit(
            name,
            lower.as_deref(),
            upper.as_deref(),
            out,
            current_size,
            writer,
        )?,
        Expr::Integral { kind } => {
            write_integral(*kind, out, writer)?;
            WriteState {
                size: current_size,
                color: ColorState::Black,
            }
        }
        Expr::IntegralOp {
            kind,
            lower,
            upper,
            body,
        } => write_integral_op(
            *kind,
            body.as_deref(),
            lower.as_deref(),
            upper.as_deref(),
            out,
            current_size,
            writer,
        )?,
        Expr::Pile { kind, upper, lower } => {
            write_pile(*kind, upper, lower, out, current_size, writer)?
        }
        Expr::Brace {
            kind,
            content,
            annotation,
        } => write_brace_template(
            *kind,
            content,
            annotation.as_deref(),
            out,
            current_size,
            writer,
        )?,
        Expr::Stackrel { upper, lower } => write_stackrel(upper, lower, out, current_size, writer)?,
        Expr::Underset { lower, base } => write_underset(lower, base, out, current_size, writer)?,
        Expr::XArrow { kind, label, under } => {
            write_xarrow(*kind, label, under.as_deref(), out, current_size, writer)?
        }
        Expr::Substack { .. } => {
            write_text("(Tex translation failed)", out)?;
            WriteState {
                size: current_size,
                color: ColorState::Black,
            }
        }
        Expr::Subarray { rows, .. } => {
            // Keep current subarray rendering on the MATRIX path until the remaining
            // MathType mixed raw/native limit layout is fully modeled.
            write_matrix(MatrixKind::Plain, rows, out, current_size, writer)?
        }
        Expr::Matrix { kind, rows } => write_matrix(*kind, rows, out, current_size, writer)?,
        Expr::Environment { kind, rows, trivia } => {
            write_environment(*kind, rows, trivia, out, current_size, writer)?
        }
        Expr::Delimited {
            left,
            right,
            content,
        } => write_delimited(*left, *right, content, out, current_size, writer)?,
        Expr::Script { base, sub, sup } => write_script(
            base,
            sub.as_deref(),
            sup.as_deref(),
            out,
            current_size,
            writer,
        )?,
    };
    Ok(next_state)
}

/// Write the color record shape MathType emits for supported \color commands.
fn write_color_expr(
    name: &str,
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    match name {
        "blue" => {
            out.extend_from_slice(&[0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe8, 0x03]);
            let selector = if writer.emit_top_color_selector_one {
                writer.emit_top_color_selector_one = false;
                0x01
            } else {
                0x02
            };
            out.extend_from_slice(&[0x0f, selector]);
            let state = write_expr(expr, out, current_size, writer)?;
            Ok(WriteState {
                size: state.size,
                color: ColorState::Default,
            })
        }
        _ => write_expr(expr, out, current_size, writer),
    }
}

/// Return true for nodes that begin by selecting their own color.
fn expr_sets_own_color(expr: &Expr) -> bool {
    match expr {
        Expr::Color { .. } | Expr::RawTex(_) => true,
        Expr::Pile {
            kind: PileKind::Binom,
            ..
        } => true,
        Expr::Style { content, .. } => expr_sets_own_color(content),
        Expr::Sequence(items) => items.first().is_some_and(expr_sets_own_color),
        _ => false,
    }
}

#[derive(Clone, Copy)]
struct ResolvedCharStyle {
    typeface: u8,
    mtcode: u16,
    font_pos: Option<u8>,
    explicit_font: Option<ExplicitFont>,
}

/// Write one MTEF CHAR record using MathType's simple font/style choices.
fn write_char(ch: char, out: &mut Vec<u8>, writer: &mut MtefWriter) -> Result<(), String> {
    let style = resolve_char_style(ch)?;
    write_styled_table_char(
        style.typeface,
        style.mtcode,
        style.font_pos,
        style.explicit_font,
        out,
        writer,
    );
    Ok(())
}

/// Resolve one visible character into the same MathType CHAR style used by both plain and accented output.
fn resolve_char_style(ch: char) -> Result<ResolvedCharStyle, String> {
    if let Some(special) = encoding::special_char(ch) {
        return Ok(ResolvedCharStyle {
            typeface: special.typeface,
            mtcode: special.mtcode,
            font_pos: special.font_pos,
            explicit_font: special.explicit_font,
        });
    }

    let code = ch as u32;
    if code > u16::MAX as u32 {
        return Err(format!(
            "character is outside BMP and not yet supported: {ch}"
        ));
    }

    if let Some(operator) = encoding::operator_char(ch) {
        return Ok(ResolvedCharStyle {
            typeface: operator.typeface,
            mtcode: operator.mtcode,
            font_pos: operator.font_pos,
            explicit_font: None,
        });
    }

    Ok(ResolvedCharStyle {
        typeface: if is_function_char(ch) {
            FN_FUNCTION
        } else if is_math_symbol_char(ch) {
            FN_SYMBOL
        } else if ch.is_ascii_digit() {
            FN_NUMBER
        } else if ch == '?' {
            // MathType falls back to a visible punctuation glyph when TeX Input cannot map
            // a direct non-ASCII literal into a math character.
            FN_FUNCTION
        } else {
            FN_VARIABLE
        },
        mtcode: code as u16,
        font_pos: None,
        explicit_font: None,
    })
}

/// Write a source-command-specific CHAR record when Unicode alone is ambiguous.
fn write_command_symbol(
    command: &str,
    ch: char,
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    write_command_symbol_with_embellishments(command, ch, &[], out, writer)
}

/// Write a source-command-specific CHAR record with optional EMBELL records.
fn write_command_symbol_with_embellishments(
    command: &str,
    ch: char,
    embellishments: &[u8],
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    let symbol = encoding::command_specific_char(command)
        .ok_or_else(|| format!("missing generated command-specific symbol: \\{command}"))?;
    if symbol.ch != ch {
        return Err(format!(
            "generated command-specific symbol mismatch for \\{command}: expected {ch}, got {}",
            symbol.ch
        ));
    }
    if embellishments.is_empty() {
        write_styled_table_char(
            symbol.typeface,
            symbol.mtcode,
            symbol.font_pos,
            symbol.explicit_font,
            out,
            writer,
        );
    } else {
        write_styled_table_char_with_embellishments(
            symbol.typeface,
            symbol.mtcode,
            symbol.font_pos,
            symbol.explicit_font,
            embellishments,
            out,
            writer,
        );
    }
    Ok(())
}

/// Write one generated CHAR entry, defining explicit Euclid fonts on demand.
fn write_styled_table_char(
    typeface: u8,
    mtcode: u16,
    font_pos: Option<u8>,
    explicit_font: Option<ExplicitFont>,
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) {
    match explicit_font {
        Some(ExplicitFont::EuclidMathOne) => {
            writer.ensure_euclid_math_one(out);
            write_table_char(writer.euclid_math_one_typeface, mtcode, font_pos, out);
        }
        Some(ExplicitFont::EuclidMathTwo) => {
            writer.ensure_euclid_math_two(out);
            write_table_char(writer.euclid_math_two_typeface, mtcode, font_pos, out);
        }
        _ => write_table_char(typeface, mtcode, font_pos, out),
    }
}

/// Write one generated CHAR entry with attached EMBELL records, defining explicit Euclid fonts on demand.
fn write_styled_table_char_with_embellishments(
    typeface: u8,
    mtcode: u16,
    font_pos: Option<u8>,
    explicit_font: Option<ExplicitFont>,
    embellishments: &[u8],
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) {
    match explicit_font {
        Some(ExplicitFont::EuclidMathOne) => {
            writer.ensure_euclid_math_one(out);
            write_table_char_with_embellishments(
                writer.euclid_math_one_typeface,
                mtcode,
                font_pos,
                embellishments,
                out,
            );
        }
        Some(ExplicitFont::EuclidMathTwo) => {
            writer.ensure_euclid_math_two(out);
            write_table_char_with_embellishments(
                writer.euclid_math_two_typeface,
                mtcode,
                font_pos,
                embellishments,
                out,
            );
        }
        _ => write_table_char_with_embellishments(typeface, mtcode, font_pos, embellishments, out),
    }
}

/// Write MathType's fnSPACE character used for spacing commands.
fn write_space(width: u8, out: &mut Vec<u8>) {
    color_default(out);
    out.extend_from_slice(&[0x02, 0x00, FN_SPACE, width, 0xef]);
}

/// Write a function-name sequence, marking the first character as function start.
fn write_function_name(
    name: &str,
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    if let Some((head, tail)) = split_lim_family_function_name(name) {
        write_function_name_chars(head, out)?;
        writer.ensure_black_color_def(out);
        color_black(out);
        write_function_name_chars(tail, out)?;
        return Ok(());
    }
    write_function_name_chars(name, out)
}

/// Write one plain function-name run without any MathType family-specific splitting.
fn write_function_name_chars(name: &str, out: &mut Vec<u8>) -> Result<(), String> {
    for (index, ch) in name.chars().enumerate() {
        let code = ch as u32;
        if code > u16::MAX as u32 {
            return Err(format!("function name character is outside BMP: {ch}"));
        }
        out.push(0x02);
        out.push(if index == 0 { 0x02 } else { 0x00 });
        out.push(FN_FUNCTION);
        write_u16(code as u16, out);
    }
    Ok(())
}

/// Return the MathType-specific split for non-template limit-family names.
fn split_lim_family_function_name(name: &str) -> Option<(&'static str, &'static str)> {
    match name {
        "liminf" => Some(("lim", "inf")),
        "limsup" => Some(("lim", "sup")),
        "injlim" => Some(("lim", "inj")),
        "projlim" => Some(("lim", "proj")),
        _ => None,
    }
}

/// Write plain text characters with MathType's text style.
fn write_text(text: &str, out: &mut Vec<u8>) -> Result<(), String> {
    write_text_code_units(text, 0x00, out)
}

/// Write characters MathType preserves from unsupported TeX syntax.
fn write_raw_tex_text(text: &str, out: &mut Vec<u8>) -> Result<(), String> {
    write_text_code_units(text, 0x80, out)
}

/// Write a TeX style switch using MathType's documented logical size records.
fn write_style_expr(
    kind: StyleKind,
    content: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let content = unwrap_single_sequence(content);
    let target_size = style_size(kind);
    let changed_size = target_size != current_size;
    let suppress_restore = writer.suppress_next_style_restore;
    if suppress_restore {
        writer.suppress_next_style_restore = false;
    }
    if kind == StyleKind::Text {
        match content {
            Expr::Fraction(numerator, denominator) => {
                return write_textstyle_fraction(
                    numerator,
                    denominator,
                    out,
                    current_size,
                    suppress_restore,
                    writer,
                );
            }
            Expr::Pile {
                kind: PileKind::Parenthesized,
                upper,
                lower,
            } => {
                return write_textstyle_parenthesized_pile(
                    upper,
                    lower,
                    out,
                    current_size,
                    suppress_restore,
                    writer,
                );
            }
            Expr::Pile {
                kind: PileKind::Binom,
                upper,
                lower,
            } => {
                return write_textstyle_binom_pile(
                    upper,
                    lower,
                    out,
                    current_size,
                    suppress_restore,
                    writer,
                );
            }
            Expr::BigOp {
                kind,
                body: None,
                lower,
                upper,
            } if lower.is_some() || upper.is_some() => {
                return write_textstyle_bodyless_big_op(
                    *kind,
                    lower.as_deref(),
                    upper.as_deref(),
                    out,
                    current_size,
                    suppress_restore,
                    writer,
                );
            }
            _ => {}
        }
    }
    let display_style_is_transparent = matches!(
        content,
        Expr::BigOp {
            body: None,
            lower,
            upper,
            ..
        } if lower.is_some() || upper.is_some()
    ) || matches!(
        content,
        Expr::Pile {
            kind: PileKind::Parenthesized | PileKind::Binom,
            ..
        }
    );
    if kind == StyleKind::Display && suppress_restore && display_style_is_transparent {
        let state = write_expr(content, out, current_size, writer)?;
        return Ok(WriteState {
            size: current_size,
            color: state.color,
        });
    }
    if suppress_restore
        && !matches!(content, Expr::Sequence(_))
        && !expr_starts_with_self_opening(content)
    {
        writer.ensure_black_color_def(out);
        color_black(out);
    }
    if changed_size && !suppress_restore {
        write_size(target_size, out);
    }
    let state = write_expr(content, out, target_size, writer)?;
    if changed_size && !suppress_restore && state.size != current_size {
        write_size(current_size, out);
    }
    Ok(WriteState {
        size: current_size,
        color: state.color,
    })
}

/// Peel one-item sequences introduced by style switches so writer rules can
/// reason about the actual styled construct.
fn unwrap_single_sequence(mut expr: &Expr) -> &Expr {
    while let Expr::Sequence(items) = expr {
        if let [item] = items.as_slice() {
            expr = item;
        } else {
            break;
        }
    }
    expr
}

/// Write MathType's text-style fraction template variant used by \tfrac.
fn write_textstyle_fraction(
    numerator: &Expr,
    denominator: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    suppress_restore: bool,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    // MathType selects black before opening the text-style fraction template,
    // even when the surrounding text-style wrapper suppresses size restoration.
    writer.ensure_black_color_def(out);
    color_black(out);
    let inner_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    out.extend_from_slice(&[0x03, 0x00, 0x0b, 0x01, 0x00]);
    write_size(inner_size, out);
    color_default(out);
    let numerator_state = write_line(numerator, out, inner_size, writer)?;
    if numerator_state.size != inner_size {
        write_size(inner_size, out);
        if numerator_state.color != ColorState::Default {
            color_default(out);
        }
    } else {
        color_default(out);
    }
    let denominator_state = write_line(denominator, out, inner_size, writer)?;
    out.push(0x00);
    if !suppress_restore && inner_size != current_size {
        write_size(current_size, out);
    }
    Ok(WriteState {
        size: current_size,
        color: denominator_state.color,
    })
}

/// Write MathType's text-style parenthesized binomial layout used by \tbinom.
fn write_textstyle_parenthesized_pile(
    upper: &Expr,
    lower: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    suppress_restore: bool,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let pile_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    write_size(pile_size, out);
    let state = write_pile(
        PileKind::Parenthesized,
        upper,
        lower,
        out,
        pile_size,
        writer,
    )?;
    if !suppress_restore && pile_size != current_size {
        write_size(current_size, out);
    }
    Ok(WriteState {
        size: current_size,
        color: state.color,
    })
}

/// Write MathType's command-form binomial layout used by \binom and \dbinom.
fn write_binom_pile(
    upper: &Expr,
    lower: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x03, 0x00, 0x01, 0x03, 0x00]);
    writer.ensure_black_color_def(out);
    color_black(out);
    out.extend_from_slice(&[0x04, 0x00, 0x02, 0x01]);
    color_default(out);
    let upper_state = write_line(upper, out, current_size, writer)?;
    if upper_state.size != current_size {
        write_size(current_size, out);
    }
    color_default(out);
    let lower_state = write_line(lower, out, current_size, writer)?;
    if lower_state.size != current_size {
        write_size(current_size, out);
    }
    out.push(0x00);
    color_default(out);
    write_delimiter_glyph_pair('(', ')', out)?;
    out.push(0x00);
    Ok(WriteState {
        size: current_size,
        color: ColorState::Default,
    })
}

/// Write MathType's text-style command-form binomial layout used by \tbinom.
fn write_textstyle_binom_pile(
    upper: &Expr,
    lower: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    suppress_restore: bool,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let pile_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    write_size(pile_size, out);
    let state = write_binom_pile(upper, lower, out, pile_size, writer)?;
    if !suppress_restore && pile_size != current_size {
        write_size(current_size, out);
    }
    Ok(WriteState {
        size: current_size,
        color: state.color,
    })
}

/// Write inline/text-style bodyless big operators, which MathType encodes with tmINTOP.
fn write_textstyle_bodyless_big_op(
    kind: BigOpKind,
    lower: Option<&Expr>,
    upper: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    suppress_restore: bool,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let script_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    let variation = if upper.is_some() { 0x30 } else { 0x10 };
    out.extend_from_slice(&[0x03, 0x00, 0x15, variation, 0x00]);
    write_null_line(out);
    write_size(script_size, out);
    if let Some(lower) = lower {
        let lower_state = write_line(lower, out, script_size, writer)?;
        restore_script_separator(lower_state, script_size, out);
    } else {
        write_null_line(out);
    }
    if let Some(upper) = upper {
        write_line(upper, out, script_size, writer)?;
    } else {
        write_null_line(out);
    }
    out.push(0x0d);
    color_default(out);
    write_named_big_operator_glyph_line(bodyless_big_op_glyph_name(kind), out, writer)?;
    out.push(0x00);
    if !suppress_restore && script_size != current_size {
        write_size(current_size, out);
    }
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
}

/// Map parser style switches onto the MTEF logical sizes already used for scripts.
fn style_size(kind: StyleKind) -> SizeState {
    match kind {
        StyleKind::Display | StyleKind::Text => SizeState::Full,
        StyleKind::Script => SizeState::Sub,
        StyleKind::ScriptScript => SizeState::Sub2,
    }
}

/// Write text-like CHAR records using the same visible fallback MathType applies.
///
/// Helper probes show that MathType text mode:
/// - turns unsupported characters into visible `?` glyphs,
/// - uses one `?` per UTF-16 code unit for non-BMP scalars,
/// - and renders visible punctuation such as `$` / `?` with function style.
fn write_text_code_units(text: &str, options: u8, out: &mut Vec<u8>) -> Result<(), String> {
    for ch in text.chars() {
        if options == 0x00 {
            let encoded = encode_mathtype_text(&ch.to_string())?;
            if encoded.iter().all(|byte| *byte == b'?') && ch != '?' {
                for _ in 0..encoded.len() {
                    write_text_char_record('?', options, FN_TEXT, out);
                }
                continue;
            }
        }
        let mut units = [0u16; 2];
        for code in ch.encode_utf16(&mut units) {
            let typeface = if options == 0x00 && text_function_style_char(ch) {
                FN_FUNCTION
            } else {
                FN_TEXT
            };
            out.push(0x02);
            out.push(options);
            out.push(typeface);
            write_u16(*code, out);
        }
    }
    Ok(())
}

/// Return true when visible text-mode punctuation uses MathType's function style.
fn text_function_style_char(ch: char) -> bool {
    matches!(ch, '$' | '?')
}

/// Write one text-mode visible glyph with an explicitly selected typeface.
fn write_text_char_record(ch: char, options: u8, typeface: u8, out: &mut Vec<u8>) {
    out.push(0x02);
    out.push(options);
    out.push(typeface);
    write_u16(ch as u16, out);
}

/// Write MathType's limit template for \lim and \sup with lower/upper slots.
fn write_limit(
    name: &str,
    lower: Option<&Expr>,
    upper: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let lower = lower.ok_or_else(|| format!("\\{name} requires a lower limit in this subset"))?;
    out.extend_from_slice(&[0x03, 0x00, 0x17, 0x10, 0x00]);
    let main = Expr::FunctionName(name.to_string());
    let main_state = write_line(&main, out, current_size, writer)?;
    let limit_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    if main_state.size != limit_size {
        write_size(limit_size, out);
    }
    color_default(out);
    let lower_state = write_line(lower, out, limit_size, writer)?;
    if lower_state.size != limit_size {
        write_size(limit_size, out);
    }
    color_default(out);
    if let Some(upper) = upper {
        write_line(upper, out, limit_size, writer)?;
    } else {
        write_null_line(out);
    }
    out.push(0x00);
    let suppress_restore = writer.suppress_next_limit_restore;
    if suppress_restore {
        writer.suppress_next_limit_restore = false;
    } else {
        if current_size != limit_size {
            write_size(current_size, out);
        }
        color_black(out);
    }
    Ok(WriteState {
        size: if suppress_restore {
            limit_size
        } else {
            current_size
        },
        color: if suppress_restore {
            ColorState::Default
        } else {
            ColorState::Black
        },
    })
}

/// Write MathType's nth-root template with index and radicand slots.
fn write_nth_root(
    index: &Expr,
    radicand: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x03, 0x00, 0x0a, 0x01, 0x00]);
    color_default(out);
    let radicand_state = write_line(radicand, out, current_size, writer)?;
    let index_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    if radicand_state.size != index_size {
        write_size(index_size, out);
    }
    color_default(out);
    let index_state = write_line(index, out, index_size, writer)?;
    out.push(0x00);
    Ok(index_state)
}

/// Write MathType's integral template with body, limit slots, and integral glyph.
fn write_integral_op(
    kind: IntegralKind,
    body: Option<&Expr>,
    lower: Option<&Expr>,
    upper: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let body = body.ok_or_else(|| "integrals require an operand in this subset".to_string())?;
    let variation = integral_variation(kind, lower.is_some() || upper.is_some());
    out.extend_from_slice(&[0x03, 0x00, 0x0f, variation, 0x00]);
    color_default(out);
    let body_state = write_line(body, out, current_size, writer)?;
    let limit_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    if body_state.size != limit_size {
        write_size(limit_size, out);
    }
    if lower.is_some() && body_state.color != ColorState::Default {
        color_default(out);
    }
    if let Some(lower) = lower {
        let lower_state = write_line(lower, out, limit_size, writer)?;
        if lower_state.size != limit_size {
            write_size(limit_size, out);
        }
        if lower_state.color != ColorState::Black {
            color_black(out);
        }
    } else if body_state.color != ColorState::Black {
        color_black(out);
    }
    if lower.is_none() {
        write_null_line(out);
    }
    if let Some(upper) = upper {
        write_line(upper, out, limit_size, writer)?;
    } else {
        write_null_line(out);
    }
    out.push(0x0d);
    if integral_has_loop(kind) {
        write_named_big_operator_glyph("contour_loop", out)?;
    }
    // Multiple-integral templates store one trailing integral glyph per sign.
    for _ in 0..integral_count(kind) {
        write_integral_glyph(out)?;
    }
    out.push(0x00);
    Ok(WriteState {
        size: limit_size,
        color: ColorState::Black,
    })
}

/// Return MathType's tmINTEG variation byte for count, loop, and limit slots.
fn integral_variation(kind: IntegralKind, has_limits: bool) -> u8 {
    let mut variation = integral_count(kind);
    if integral_has_loop(kind) {
        variation |= 0x04;
    }
    if has_limits {
        variation |= 0x10;
    }
    variation
}

/// Return how many integral signs a MathType integral template carries.
fn integral_count(kind: IntegralKind) -> u8 {
    match kind {
        IntegralKind::Single | IntegralKind::Contour => 1,
        IntegralKind::Double => 2,
        IntegralKind::Triple => 3,
    }
}

/// Return true for contour integral variants that add MathType's loop glyph.
fn integral_has_loop(kind: IntegralKind) -> bool {
    matches!(kind, IntegralKind::Contour)
}

/// Write the integral glyph MathType appends at the end of integral templates.
fn write_integral_glyph(out: &mut Vec<u8>) -> Result<(), String> {
    write_named_big_operator_glyph("integral", out)
}

/// Write a generated glyph used by MathType's big-operator templates.
fn write_named_big_operator_glyph(name: &str, out: &mut Vec<u8>) -> Result<(), String> {
    let glyph = encoding::big_operator_glyph(name)?;
    if let Some(selector) = glyph.font_style_selector {
        out.extend_from_slice(&[0x08, selector, 0x00]);
    }
    write_styled_table_char(
        glyph.typeface,
        glyph.mtcode,
        Some(glyph.font_pos),
        glyph.explicit_font,
        out,
        &mut MtefWriter {
            euclid_math_one_defined: false,
            euclid_math_one_typeface: EXPLICIT_FONT_NEG_1,
            euclid_math_two_defined: false,
            euclid_math_two_typeface: EXPLICIT_FONT_NEG_1,
            euclid_fraktur_defined: false,
            euclid_fraktur_typeface: EXPLICIT_FONT_NEG_1,
            sans_serif_defined: false,
            sans_serif_typeface: EXPLICIT_FONT_NEG_1,
            black_color_defined: false,
            sans_serif_group_active: false,
            typewriter_group_active: false,
            big_symbol_line_marker_pending: false,
            suppress_next_pile_color_default: false,
            suppress_next_stackrel_color_default: false,
            emit_top_fenced_matrix_color: false,
            suppress_next_style_restore: false,
            suppress_next_limit_restore: false,
            emit_top_color_selector_one: false,
            top_sequence_starts_default: false,
            fallback_environment_active: false,
            suppress_next_line_black: false,
            line_starts_default: false,
            parent_sequence_has_previous_sibling: false,
        },
    );
    Ok(())
}

/// Write a standalone integral glyph when no template body follows.
fn write_integral(
    kind: IntegralKind,
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    if let Some(command) = standalone_integral_command(kind) {
        let symbol = encoding::command_specific_char(command)
            .ok_or_else(|| format!("missing generated standalone integral symbol: \\{command}"))?;
        write_styled_table_char(
            symbol.typeface,
            symbol.mtcode,
            symbol.font_pos,
            symbol.explicit_font,
            out,
            writer,
        );
        return Ok(());
    }
    if integral_has_loop(kind) {
        write_named_big_operator_glyph("contour_loop", out)?;
    }
    for _ in 0..integral_count(kind) {
        write_integral_glyph(out)?;
    }
    Ok(())
}

/// Return the source command whose standalone integral glyph is generated from MathType probes.
fn standalone_integral_command(kind: IntegralKind) -> Option<&'static str> {
    match kind {
        IntegralKind::Single => Some("int"),
        IntegralKind::Contour => Some("oint"),
        IntegralKind::Double => Some("iint"),
        IntegralKind::Triple => Some("iiint"),
    }
}

/// Write a font-scoped expression for the MathType font commands used here.
fn write_font_expr(
    kind: FontKind,
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if kind == FontKind::MathSf && expr_starts_with_font_wrapper(expr) {
        writer.ensure_black_color_def(out);
        color_black(out);
        return write_expr(expr, out, current_size, writer);
    }
    let opened_sans_serif_group = kind == FontKind::MathSf && !writer.sans_serif_group_active;
    let opened_typewriter_group =
        kind == FontKind::TypewriterText && !writer.typewriter_group_active;
    if opened_sans_serif_group {
        writer.ensure_sans_serif(out);
        writer.ensure_black_color_def(out);
        color_black(out);
        writer.sans_serif_group_active = true;
    }
    if opened_typewriter_group {
        // MathType's native \texttt opens its Courier-like text style with a
        // small FONT_STYLE_DEF marker before the visible CHAR records.
        out.extend_from_slice(&[0x08, 0x03, 0x00]);
        writer.ensure_black_color_def(out);
        color_black(out);
        writer.typewriter_group_active = true;
    }
    let result = match expr {
        Expr::Sequence(items) => {
            let mut state = WriteState {
                size: current_size,
                color: ColorState::Black,
            };
            for item in items {
                if state.size != current_size {
                    write_size(current_size, out);
                    state.size = current_size;
                }
                if state.color != ColorState::Black {
                    color_black(out);
                    state.color = ColorState::Black;
                }
                state = write_font_expr(kind, item, out, state.size, writer)?;
            }
            Ok(state)
        }
        Expr::Char(ch) => {
            write_font_char(kind, *ch, out, writer)?;
            Ok(WriteState {
                size: current_size,
                color: ColorState::Black,
            })
        }
        Expr::Accent {
            kind: accent_kind,
            content,
        } if kind == FontKind::Bold
            && matches!(accent_kind, AccentKind::Hat | AccentKind::WideHat)
            && single_accent_char(content).is_some() =>
        {
            let ch = single_accent_char(content).expect("guard checked single accent char");
            write_bold_embellished_char(ch, *accent_kind, out)?;
            Ok(WriteState {
                size: current_size,
                color: ColorState::Black,
            })
        }
        Expr::Script { base, sub, sup } => {
            // MathType keeps font-scoped scripts by applying the font to each visible script slot
            // instead of dropping back to an unscoped native Script record.
            let scripted = Expr::Script {
                base: Box::new(font_wrapped_expr(kind, base.as_ref())),
                sub: sub
                    .as_deref()
                    .map(|expr| Box::new(font_wrapped_expr(kind, expr))),
                sup: sup
                    .as_deref()
                    .map(|expr| Box::new(font_wrapped_expr(kind, expr))),
            };
            write_expr(&scripted, out, current_size, writer)
        }
        other => write_expr(other, out, current_size, writer),
    };
    if opened_sans_serif_group {
        writer.sans_serif_group_active = false;
    }
    if opened_typewriter_group {
        writer.typewriter_group_active = false;
    }
    result
}

/// Return true when nested content already chooses its own math-font wrapper.
fn expr_starts_with_font_wrapper(expr: &Expr) -> bool {
    match expr {
        Expr::Font { .. } => true,
        Expr::Style { content, .. } => expr_starts_with_font_wrapper(content),
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_font_wrapper),
        _ => false,
    }
}

/// Write one character under a LaTeX math font command.
fn write_font_char(
    kind: FontKind,
    ch: char,
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    let code = ch as u32;
    if code > u16::MAX as u32 {
        return Err(format!("font character is outside BMP: {ch}"));
    }
    match kind {
        FontKind::Bold => {
            if !char_prefers_bold_font(ch) {
                return write_char(ch, out, writer);
            }
            out.push(0x02);
            out.push(0x00);
            out.push(FN_VECTOR);
            write_u16(code as u16, out);
        }
        FontKind::RomanText => {
            write_table_char(FN_TEXT, code as u16, None, out);
        }
        FontKind::TypewriterText => {
            // MathType's native \texttt uses the active explicit text-style
            // font slot (-1 biased to 0x7f), not fnUSER1.
            write_table_char(EXPLICIT_FONT_NEG_1, code as u16, None, out);
        }
        FontKind::MathCal => {
            let Ok(entry) = encoding::mathcal_char(ch) else {
                return write_non_alpha_font_char(kind, ch, out, writer);
            };
            if entry.font_pos.is_some() {
                writer.ensure_euclid_math_one(out);
                write_table_char(
                    writer.euclid_math_one_typeface,
                    entry.mtcode,
                    entry.font_pos,
                    out,
                );
            } else {
                write_table_char(entry.typeface, entry.mtcode, entry.font_pos, out);
            }
        }
        FontKind::MathScr => {
            let Ok(entry) = encoding::mathscr_char(ch) else {
                return write_non_alpha_font_char(kind, ch, out, writer);
            };
            write_math_one_font_char(entry, out, writer);
        }
        FontKind::MathFrak => {
            let Ok(entry) = encoding::mathfrak_char(ch) else {
                return write_non_alpha_font_char(kind, ch, out, writer);
            };
            if entry.font_pos.is_some() && entry.typeface == EXPLICIT_FONT_NEG_1 {
                writer.ensure_euclid_fraktur(out);
                write_table_char(
                    writer.euclid_fraktur_typeface,
                    entry.mtcode,
                    entry.font_pos,
                    out,
                );
            } else {
                write_table_char(entry.typeface, entry.mtcode, entry.font_pos, out);
            }
        }
        FontKind::MathSf => {
            writer.ensure_sans_serif(out);
            write_table_char(writer.sans_serif_typeface, code as u16, None, out);
        }
        FontKind::MathBb => {
            let Ok(entry) = encoding::mathbb_char(ch) else {
                return write_non_alpha_font_char(kind, ch, out, writer);
            };
            if entry.font_pos.is_some() && entry.typeface == EXPLICIT_FONT_NEG_1 {
                writer.ensure_euclid_math_two(out);
                write_table_char(
                    writer.euclid_math_two_typeface,
                    entry.mtcode,
                    entry.font_pos,
                    out,
                );
            } else {
                // MathType stores C/N/Q/R/Z blackboard letters through fnMTEXTRA
                // instead of the Euclid Math Two explicit font definition.
                debug_assert_eq!(entry.typeface, FN_MT_EXTRA);
                write_table_char(entry.typeface, entry.mtcode, entry.font_pos, out);
            }
        }
    }
    Ok(())
}

/// Wrap one expression in a font node while avoiding duplicate wrappers.
fn font_wrapped_expr(kind: FontKind, expr: &Expr) -> Expr {
    match expr {
        Expr::Font { kind: existing, .. } if *existing == kind => expr.clone(),
        _ => Expr::Font {
            kind,
            content: Box::new(expr.clone()),
        },
    }
}

/// Return true when MathType really uses the bold math font instead of the regular glyph slot.
fn char_prefers_bold_font(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '\u{03b1}'..='\u{03c9}' | '\u{0391}'..='\u{03a9}')
}

/// Write a generated character that uses MathType's Euclid Math One definition.
fn write_math_one_font_char(
    entry: crate::generated::char_tables::EncodedChar,
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) {
    writer.ensure_euclid_math_one(out);
    write_table_char(
        writer.euclid_math_one_typeface,
        entry.mtcode,
        entry.font_pos,
        out,
    );
}

/// Fall back for digits and punctuation inside alphabet-only math font commands.
fn write_non_alpha_font_char(
    kind: FontKind,
    ch: char,
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    if ch.is_ascii_alphabetic() {
        return Err(format!("unsupported {kind:?} character: {ch}"));
    }
    write_char(ch, out, writer)
}

/// Write a non-null LINE record with MathType's black color selection inside.
fn write_line(
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x01, 0x00]);
    if let Some((width, rest)) = leading_space_rest(expr) {
        write_space_without_color(width, out);
        if !rest.is_empty() {
            color_black(out);
            let rest_expr = Expr::Sequence(rest.to_vec());
            let final_state = write_expr(&rest_expr, out, current_size, writer)?;
            out.push(0x00);
            return Ok(final_state);
        }
        out.push(0x00);
        return Ok(WriteState {
            size: current_size,
            color: ColorState::Default,
        });
    }
    let starts_with_euclid_math_one = expr_starts_with_euclid_math_one(expr);
    let starts_with_euclid_math_two = expr_starts_with_euclid_math_two(expr);
    let starts_with_euclid_fraktur = expr_starts_with_euclid_fraktur(expr);
    let suppress_black = writer.suppress_next_line_black;
    if suppress_black {
        writer.suppress_next_line_black = false;
    }
    if starts_with_euclid_math_one {
        writer.ensure_euclid_math_one(out);
    } else if starts_with_euclid_math_two {
        writer.ensure_euclid_math_two(out);
    } else if starts_with_euclid_fraktur {
        writer.ensure_euclid_fraktur(out);
    }
    if expr_starts_with_big_symbol_script_base(expr) {
        out.push(0x0d);
        writer.big_symbol_line_marker_pending = true;
    } else if expr_is_standalone_limit(expr) {
        // Standalone \lim/\sup probes keep the template's internal size/color state visible
        // through the end of the LINE instead of restoring it for a following sibling.
        writer.suppress_next_limit_restore = true;
    } else if expr_starts_with_standalone_integral(expr)
        || expr_starts_with_standalone_big_glyph(expr)
    {
        out.push(0x0d);
    }
    let line_starts_default = suppress_black || expr_starts_with_self_opening(expr);
    if !suppress_black && !expr_starts_with_self_opening(expr) {
        writer.ensure_black_color_def(out);
        color_black(out);
    }
    let previous_line_starts_default = writer.line_starts_default;
    let previous_top_sequence_starts_default = writer.top_sequence_starts_default;
    writer.line_starts_default = line_starts_default;
    if line_starts_default && matches!(expr, Expr::Sequence(_)) {
        writer.top_sequence_starts_default = true;
    }
    writer.suppress_next_stackrel_color_default = expr_starts_with_stackrel(expr);
    let final_state = write_expr(expr, out, current_size, writer)?;
    writer.line_starts_default = previous_line_starts_default;
    writer.top_sequence_starts_default = previous_top_sequence_starts_default;
    out.push(0x00);
    Ok(final_state)
}
