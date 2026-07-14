use crate::ast::*;
use crate::generated::char_tables::ExplicitFont;
use crate::generated::color_tables::named_color_def;
use crate::mathtype_ansi::encode_mathtype_text;
use crate::typeface::{
    EXPLICIT_FONT_NEG_1, EXPLICIT_FONT_NEG_2, FN_FUNCTION, FN_MT_EXTRA, FN_NUMBER, FN_SPACE,
    FN_SYMBOL, FN_TEXT, FN_VARIABLE, FN_VECTOR,
};
use std::collections::HashMap;

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
#[path = "mtef/source.rs"]
mod source;
#[path = "mtef/templates.rs"]
mod templates;

use accents::*;
use delimiters::*;
use environments::*;
use predicates::*;
use records::{
    color_black, color_default, write_expanding_glyph, write_null_line, write_size,
    write_table_char, write_table_char_with_embellishments, write_u16,
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
    suppress_next_pile_black_selector: bool,
    suppress_next_stackrel_color_default: bool,
    emit_top_fenced_matrix_color: bool,
    suppress_next_style_restore: bool,
    force_next_no_limits_integral_style_restore: bool,
    suppress_next_limit_restore: bool,
    emit_top_color_selector_one: bool,
    next_color_selector: u8,
    named_color_selectors: HashMap<String, u8>,
    pending_raw_follow_selector: Option<u8>,
    top_sequence_starts_default: bool,
    fallback_environment_active: bool,
    suppress_next_line_black: bool,
    line_starts_default: bool,
    parent_sequence_has_previous_sibling: bool,
    parent_sequence_previous_was_raw: bool,
    suppress_next_marked_char_marker: bool,
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
    let translation_failed = expr_is_translation_failed_placeholder(expr);
    let mut out = Vec::new();
    // MathType switches to a shorter failure-form header when TeX Input collapses
    // the whole formula into "(Text translation failed)".
    source::write_header(!translation_failed, &mut out);

    if !translation_failed {
        source::write_tex_source_record(source_latex_header_text(source_latex, expr), &mut out)?;
    }

    if let Some(path) = prefs_file {
        out.extend_from_slice(&fixed_defs::fixed_defs_from_prefs_file(path)?);
    } else {
        out.extend_from_slice(fixed_defs::fixed_defs()?);
    }
    write_equation_body(expr, &mut out)?;
    Ok(out)
}

/// Recover the source LaTeX stored in MathType's TeX-source future record.
pub(crate) fn read_tex_source(mtef: &[u8]) -> Result<Option<String>, String> {
    source::read_tex_source(mtef)
}

/// Return the TeX-source string MathType stores in the future record.
///
/// Bug-fix: when the entire formula is one dropped escaped delimiter shim such as
/// `\]`, MathType omits the original source text from the header as well.
fn source_latex_header_text<'a>(source_latex: &'a str, expr: &Expr) -> &'a str {
    if expr_is_empty_sequence(expr) && is_dropped_escaped_shim_source(source_latex) {
        ""
    } else {
        source_latex
    }
}

/// Return true when the whole input is one escaped shim that MathType drops completely.
fn is_dropped_escaped_shim_source(source_latex: &str) -> bool {
    matches!(source_latex, "\\]")
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
    if matches!(expr, Expr::Sequence(items) if items.is_empty()) {
        // Bug-fix: a fully ignored formula such as `\hline` keeps only the
        // top-level size byte and final END marker; MathType does not emit an
        // empty LINE record in between.
        out.extend_from_slice(&[0x0a, 0x00]);
        return Ok(());
    }
    if expr_starts_with_top_annotated_align_pile(expr) {
        out.push(0x0a);
    } else {
        out.extend_from_slice(&[0x0a, 0x01, 0x00]);
    }
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
        suppress_next_pile_black_selector: false,
        suppress_next_stackrel_color_default: false,
        emit_top_fenced_matrix_color: false,
        suppress_next_style_restore: false,
        force_next_no_limits_integral_style_restore: false,
        suppress_next_limit_restore: false,
        emit_top_color_selector_one: false,
        next_color_selector: 1,
        named_color_selectors: HashMap::new(),
        pending_raw_follow_selector: None,
        top_sequence_starts_default: false,
        fallback_environment_active: false,
        suppress_next_line_black: false,
        line_starts_default: false,
        parent_sequence_has_previous_sibling: false,
        parent_sequence_previous_was_raw: false,
        suppress_next_marked_char_marker: false,
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
        } else if expr_starts_with_marked_char(expr) {
            out.push(0x0d);
            writer.suppress_next_marked_char_marker = true;
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

/// Return true when the equation body starts with MathType's align-family row-stack PILE form.
fn expr_starts_with_top_annotated_align_pile(expr: &Expr) -> bool {
    match expr {
        Expr::Environment {
            kind: EnvironmentKind::Align | EnvironmentKind::AlignAt | EnvironmentKind::AlignedAt,
            rows,
            ..
        } => !matches!(rows.as_slice(), [row] if matches!(row.as_slice(), [_])),
        Expr::Style { content, .. } => expr_starts_with_top_annotated_align_pile(content),
        Expr::Sequence(items) => items
            .first()
            .is_some_and(expr_starts_with_top_annotated_align_pile),
        _ => false,
    }
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
            let mut index = 0usize;
            while index < items.len() {
                let spacing_override = script_mod_spacing_expr(&items[index], current_size);
                let item = spacing_override.as_ref().unwrap_or(&items[index]);
                if index > 0
                    && !matches!(items[index - 1], Expr::RawTex(_))
                    && !(expr_starts_with_raw_tex(item)
                        && items[index - 1].is_non_black_color_expr())
                {
                    writer.pending_raw_follow_selector = None;
                }
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
                    if state.color == ColorState::Default
                        || (index > 0 && matches!(items.get(index - 1), Some(Expr::Space(_))))
                    {
                        // Bug-fix: adjacent spacing commands such as `\!\!2` stay on
                        // MathType's default-color path until the next visible glyph.
                        write_space_without_color(*width, out);
                        state = WriteState {
                            size: state.size,
                            color: ColorState::Default,
                        };
                        index += 1;
                        continue;
                    }
                }
                let needs_black_selector = !(state.color == ColorState::Black
                    || (index == 0 && start_default)
                    || expr_starts_with_line_font_def(item)
                    || expr_style_defers_leading_space_open(item)
                    || expr_starts_with_empty_base_superscript(item)
                    || expr_sets_own_color(item));
                if index > 0
                    && matches!(items.get(index - 1), Some(Expr::RawTex(raw)) if raw.starts_with("\\left"))
                    && expr_starts_with_standalone_big_glyph(item)
                {
                    // Bug-fix: raw/native hybrids such as `\left( \sum_1^n \right)^{2}`
                    // still keep MathType's line marker on the visible standalone
                    // big-operator glyph that follows the raw fence shell.
                    out.push(0x0d);
                }
                if matches!(item, Expr::MarkedChar(_) | Expr::Marked(_)) {
                    if writer.suppress_next_marked_char_marker {
                        writer.suppress_next_marked_char_marker = false;
                    } else {
                        out.push(0x0d);
                    }
                }
                if needs_black_selector {
                    if matches!(items.get(index.saturating_sub(1)), Some(Expr::RawTex(_))) {
                        if let Some(selector) = writer.pending_raw_follow_selector.take() {
                            out.extend_from_slice(&[0x0f, selector]);
                            state.color = ColorState::Black;
                        } else {
                            if expr_starts_with_euclid_math_one(item) {
                                writer.ensure_euclid_math_one(out);
                            } else if expr_starts_with_euclid_math_two(item) {
                                writer.ensure_euclid_math_two(out);
                            } else if expr_starts_with_euclid_fraktur(item) {
                                writer.ensure_euclid_fraktur(out);
                            }
                            if !raw_fallback_reuses_black_selector(items, index) {
                                writer.ensure_black_color_def(out);
                            }
                            color_black(out);
                            if writer.next_color_selector == 1 {
                                writer.next_color_selector = 2;
                            }
                            state.color = ColorState::Black;
                        }
                    } else {
                        if expr_starts_with_euclid_math_one(item) {
                            writer.ensure_euclid_math_one(out);
                        } else if expr_starts_with_euclid_math_two(item) {
                            writer.ensure_euclid_math_two(out);
                        } else if expr_starts_with_euclid_fraktur(item) {
                            writer.ensure_euclid_fraktur(out);
                        }
                        if !raw_fallback_reuses_black_selector(items, index) {
                            writer.ensure_black_color_def(out);
                        }
                        color_black(out);
                        if writer.next_color_selector == 1 {
                            writer.next_color_selector = 2;
                        }
                        state.color = ColorState::Black;
                    }
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
                if index > 0
                    && expr_starts_with_raw_tex(item)
                    && items[index - 1].is_non_black_color_expr()
                {
                    // Bug-fix: after a non-black `\color{...}` group, MathType emits
                    // an explicit default-color selector before a raw fallback segment
                    // such as `\kern`, even though the logical writer state has already
                    // transitioned away from the colored child expression.
                    color_default(out);
                    state.color = ColorState::Default;
                } else if index > 0
                    && state.color == ColorState::Black
                    && expr_starts_with_raw_tex(item)
                {
                    // Bug-fix: MathType closes a named-color run before a raw fallback
                    // fragment such as `\kern`, then reuses that selector on the next
                    // visible token that belongs to the same fallback segment.
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
                    && expr_starts_with_parenthesized_pile(item)
                {
                    // Bug-fix: inline old-TeX `\choose` piles continue the
                    // already-open visible black run instead of emitting an
                    // extra selector before the parenthesized pile template.
                    writer.suppress_next_pile_black_selector = true;
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
                let previous_parent_sequence_previous_was_raw =
                    writer.parent_sequence_previous_was_raw;
                let suppress_limit_restore_for_following_raw =
                    items.get(index + 1).is_some_and(expr_starts_with_raw_tex)
                        && expr_ends_with_limit_like_template(item);
                if suppress_limit_restore_for_following_raw {
                    // Bug-fix: MathType keeps the default-color path open when a native
                    // limit template is immediately followed by one raw `\limits` /
                    // `\nolimits` fragment on the same line.
                    writer.suppress_next_limit_restore = true;
                }
                if items.get(index + 1).is_some_and(expr_starts_with_raw_tex)
                    && matches!(
                        unwrap_single_sequence(item),
                        Expr::IntegralOp {
                            body: None,
                            placement: LimitPlacement::NoLimits,
                            ..
                        }
                    )
                {
                    // Bug-fix: side-script integrals still restore full size before one
                    // following raw `\limits_` fragment, unlike the standalone no-limits
                    // line-ending case that stays on the compact tmINTOP path.
                    writer.force_next_no_limits_integral_style_restore = true;
                }
                writer.parent_sequence_has_previous_sibling = index > 0;
                writer.parent_sequence_previous_was_raw =
                    index > 0 && matches!(items[index - 1], Expr::RawTex(_));
                if let Some(source) = prime_embellished_sequence_char(items, index) {
                    match source {
                        PrimeEmbellishedSource::Char { ch, prime_count } => {
                            write_embellished_char_codes(
                                ch,
                                &prime_embellishment_codes(prime_count),
                                out,
                                writer,
                            )?;
                            index += 1 + prime_count;
                        }
                        PrimeEmbellishedSource::Command {
                            command,
                            ch,
                            prime_count,
                        } => {
                            write_command_symbol_with_embellishments(
                                &command,
                                ch,
                                &prime_embellishment_codes(prime_count),
                                out,
                                writer,
                            )?;
                            index += 1 + prime_count;
                        }
                    }
                    state = WriteState {
                        size: state.size,
                        color: ColorState::Black,
                    };
                } else {
                    state = write_expr(item, out, state.size, writer)?;
                    index += 1;
                }
                if matches!(item, Expr::MarkedChar(_) | Expr::Marked(_)) && index < items.len() {
                    if matches!(
                        items.get(index),
                        Some(Expr::MarkedChar(_) | Expr::Marked(_))
                    ) {
                        // Bug-fix: a run of consecutive delimiter-size hints keeps just one
                        // initial line marker, so each following marked fence suppresses its
                        // own marker until the run returns to ordinary same-line content.
                        writer.suppress_next_marked_char_marker = true;
                    } else {
                        // Bug-fix: delimiter-size hints such as `\Bigg[` leave MathType's
                        // line marker on the first fence and then explicitly restore the
                        // current logical size before the remaining same-line content.
                        write_size(current_size, out);
                        state.size = current_size;
                    }
                }
                if matches!(item, Expr::SumOperatorSymbol(_) | Expr::BigSymbol(_))
                    && items.get(index).is_some_and(
                        |next| matches!(next, Expr::RawTex(raw) if raw == "^" || raw == "_"),
                    )
                {
                    // Bug-fix: bodyless big operators restore full size before the raw
                    // repeated-script marker that MathType stores after the glyph.
                    write_size(current_size, out);
                    state.size = current_size;
                }
                if display_fraction_sequence_item_needs_full_restore(item, items.get(index)) {
                    // MathType's SetData path explicitly restores full size after a display
                    // fraction when the same line continues with punctuation, as in cases cells.
                    write_size(current_size, out);
                    state.size = current_size;
                }
                writer.parent_sequence_has_previous_sibling =
                    previous_parent_sequence_has_previous_sibling;
                writer.parent_sequence_previous_was_raw = previous_parent_sequence_previous_was_raw;
            }
            state
        }
        Expr::DefaultColor(content) => {
            color_default(out);
            let state = write_expr(content, out, current_size, writer)?;
            WriteState {
                size: state.size,
                color: ColorState::Default,
            }
        }
        Expr::HybridLayout(parts) => write_hybrid_layout(parts, out, current_size, writer)?,
        Expr::Char(ch) => {
            write_char(*ch, out, writer)?;
            WriteState {
                size: current_size,
                color: ColorState::Black,
            }
        }
        Expr::MarkedChar(ch) => {
            write_char(*ch, out, writer)?;
            WriteState {
                size: current_size,
                color: ColorState::Black,
            }
        }
        Expr::Marked(content) => write_expr(content, out, current_size, writer)?,
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
        Expr::RawBoundary => {
            // Bug-fix: MathType sometimes closes one raw fallback segment and
            // immediately starts another without any visible separator, such as
            // zero-argument raw macro definitions followed by a raw replacement.
            out.extend_from_slice(&[0x00, 0x00]);
            WriteState {
                size: current_size,
                color: ColorState::Default,
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
            placement,
        } => {
            if *placement == LimitPlacement::NoLimits && (lower.is_some() || upper.is_some()) {
                let expr = no_limits_big_op_expr(
                    *kind,
                    lower.as_deref(),
                    upper.as_deref(),
                    body.as_deref(),
                );
                writer.suppress_next_style_restore = true;
                write_expr(&expr, out, current_size, writer)?
            } else {
                write_big_op(
                    *kind,
                    body.as_deref(),
                    lower.as_deref(),
                    upper.as_deref(),
                    out,
                    current_size,
                    writer,
                )?
            }
        }
        Expr::FallbackBigOp { kind, body } => {
            write_fallback_big_op_body(*kind, body, out, current_size, writer)?
        }
        Expr::Limit {
            name,
            lower,
            upper,
            placement,
        } => write_limit_expr(
            name,
            lower.as_deref(),
            upper.as_deref(),
            *placement,
            out,
            current_size,
            writer,
        )?,
        Expr::MathOp {
            content,
            lower,
            upper,
            placement,
            leading_space,
        } => write_math_op_expr(
            MathOpWrite {
                content,
                lower: lower.as_deref(),
                upper: upper.as_deref(),
                placement: *placement,
                leading_space: *leading_space,
            },
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
            placement,
        } => write_integral_expr(
            IntegralWrite {
                kind: *kind,
                body: body.as_deref(),
                lower: lower.as_deref(),
                upper: upper.as_deref(),
                placement: *placement,
            },
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
        Expr::Subarray { column_spec, rows } => {
            write_subarray_fallback(column_spec, rows, out, current_size, writer)?
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
        Expr::OneSidedDelimited {
            side,
            delimiter,
            content,
        } => write_one_sided_delimited(*side, *delimiter, content, out, current_size, writer)?,
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

/// Expand `\mod`'s leading space into the probe-backed thin-space sequence inside scripts.
fn script_mod_spacing_expr(expr: &Expr, current_size: SizeState) -> Option<Expr> {
    if current_size != SizeState::Sub {
        return None;
    }
    match expr {
        Expr::Sequence(items)
            if matches!(
                items.as_slice(),
                [Expr::Space(0x05), Expr::FunctionName(name), Expr::Space(0x02)] if name == "mod"
            ) =>
        {
            Some(Expr::Sequence(vec![
                Expr::Space(0x04),
                Expr::Space(0x04),
                Expr::Space(0x04),
                Expr::FunctionName("mod".to_string()),
                Expr::Space(0x02),
            ]))
        }
        _ => None,
    }
}

/// Return true when a style wrapper keeps its opening spacing on the default path.
fn expr_style_defers_leading_space_open(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Style {
            kind: StyleKind::Script | StyleKind::ScriptScript,
            content,
        } if leading_space_rest(content).is_some()
    )
}

/// Write MathType's hybrid fallback runs that interleave raw TeX and standalone LINE records.
fn write_hybrid_layout(
    parts: &[HybridPart],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let mut state = WriteState {
        size: current_size,
        color: ColorState::Default,
    };
    for part in parts {
        state = match part {
            HybridPart::Raw(text) => {
                if state.color != ColorState::Default {
                    color_default(out);
                }
                write_raw_tex_text(text, out)?;
                WriteState {
                    size: current_size,
                    color: ColorState::Default,
                }
            }
            HybridPart::Line(expr) => {
                if state.color != ColorState::Black && !expr_starts_with_self_opening(expr) {
                    if expr_starts_with_euclid_math_one(expr) {
                        writer.ensure_euclid_math_one(out);
                    } else if expr_starts_with_euclid_math_two(expr) {
                        writer.ensure_euclid_math_two(out);
                    } else if expr_starts_with_euclid_fraktur(expr) {
                        writer.ensure_euclid_fraktur(out);
                    }
                    writer.ensure_black_color_def(out);
                    color_black(out);
                }
                write_expr(expr, out, current_size, writer)?
            }
        };
    }
    Ok(state)
}

enum PrimeEmbellishedSource {
    Char {
        ch: char,
        prime_count: usize,
    },
    Command {
        command: String,
        ch: char,
        prime_count: usize,
    },
}

/// Return the base symbol source for MathType's compact apostrophe-prime embellishment.
fn prime_embellished_sequence_char(items: &[Expr], index: usize) -> Option<PrimeEmbellishedSource> {
    let prime_count = items[index + 1..]
        .iter()
        .take_while(|item| matches!(item, Expr::Char('\'')))
        .count();
    if prime_count == 0 {
        return None;
    }
    match items.get(index) {
        Some(Expr::Char(ch)) => Some(PrimeEmbellishedSource::Char {
            ch: *ch,
            prime_count,
        }),
        Some(Expr::CommandSymbol { command, ch }) => Some(PrimeEmbellishedSource::Command {
            command: command.clone(),
            ch: *ch,
            prime_count,
        }),
        _ => None,
    }
}

/// Return MathType's embellishment sequence for one parsed postfix prime run.
fn prime_embellishment_codes(prime_count: usize) -> Vec<u8> {
    match prime_count {
        0 => Vec::new(),
        1 => vec![EMBELL_PRIME],
        2 => vec![EMBELL_DOUBLE_PRIME],
        // Bug-fix: MathType uses a dedicated double-prime embellishment when
        // two apostrophes stay attached to the same base. Longer runs keep that
        // compact prefix and then append remaining single-prime embellishments.
        count => {
            let mut codes = vec![EMBELL_DOUBLE_PRIME];
            codes.extend(std::iter::repeat_n(EMBELL_PRIME, count - 2));
            codes
        }
    }
}

/// Return true for display fractions that MathType follows with an explicit full-size restore.
fn display_fraction_sequence_item_needs_full_restore(item: &Expr, next: Option<&Expr>) -> bool {
    matches!(
        (unwrap_single_sequence(item), next.map(unwrap_single_sequence)),
        (
            Expr::Style {
                kind: StyleKind::Display,
                content,
            },
            Some(Expr::Char(',' | '.' | ';' | ':'))
        ) if matches!(unwrap_single_sequence(content), Expr::Fraction(_, _))
    )
}

/// Write the color record shape MathType emits for supported \color commands.
fn write_color_expr(
    name: &str,
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if name == "black" {
        // Bug-fix: MathType reuses the shared black COLOR_DEF once it already
        // exists; nested `\color{black}{...}` only emits selector `0x0f 0x01`.
        writer.ensure_black_color_def(out);
        out.extend_from_slice(&[0x0f, 0x01]);
        let state = write_expr(expr, out, current_size, writer)?;
        return Ok(state);
    }
    if let Some(prefix) = named_color_def(name) {
        if let Some((leading_raw, tail)) = split_leading_raw_visible_tail(expr) {
            write_expr(&leading_raw, out, current_size, writer)?;
            let selector = write_named_color_selector(name, prefix, out, writer);
            out.extend_from_slice(&[0x0f, selector]);
            writer.pending_raw_follow_selector = Some(selector);
            let state = write_expr(&tail, out, current_size, writer)?;
            return Ok(WriteState {
                size: state.size,
                color: ColorState::Default,
            });
        }
        let selector = write_named_color_selector(name, prefix, out, writer);
        out.extend_from_slice(&[0x0f, selector]);
        writer.pending_raw_follow_selector = Some(selector);
        let state = write_expr(expr, out, current_size, writer)?;
        Ok(WriteState {
            size: state.size,
            color: ColorState::Default,
        })
    } else {
        write_expr(expr, out, current_size, writer)
    }
}

/// Emit one named color definition when needed and return the selector MathType uses for it.
fn write_named_color_selector(
    name: &str,
    prefix: &[u8],
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> u8 {
    if let Some(selector) = writer.named_color_selectors.get(name).copied() {
        return selector;
    }
    out.extend_from_slice(prefix);
    let selector = if writer.emit_top_color_selector_one && writer.next_color_selector == 1 {
        writer.emit_top_color_selector_one = false;
        0x01
    } else if writer.next_color_selector == 1 {
        writer.emit_top_color_selector_one = false;
        0x02
    } else {
        writer.emit_top_color_selector_one = false;
        writer.next_color_selector
    };
    if writer.next_color_selector <= selector {
        writer.next_color_selector = selector.saturating_add(1);
    }
    writer
        .named_color_selectors
        .insert(name.to_string(), selector);
    selector
}

/// Split one expression into a leading raw run plus the visible tail that follows it.
fn split_leading_raw_visible_tail(expr: &Expr) -> Option<(Expr, Expr)> {
    match expr {
        Expr::Sequence(items) => {
            let first = items.first()?;
            if matches!(first, Expr::RawTex(_)) {
                if items.len() < 2 {
                    return None;
                }
                return Some((
                    first.clone(),
                    collapse_single_expr(Expr::Sequence(items[1..].to_vec())),
                ));
            }
            let (leading_raw, first_tail) = split_leading_raw_visible_tail(first)?;
            let mut tail_items = match first_tail {
                Expr::Sequence(items) => items,
                other => vec![other],
            };
            tail_items.extend(items[1..].iter().cloned());
            Some((
                leading_raw,
                collapse_single_expr(Expr::Sequence(tail_items)),
            ))
        }
        _ => None,
    }
}

/// Collapse one single-item sequence so byte-shaping helpers can rebuild compact tails.
fn collapse_single_expr(expr: Expr) -> Expr {
    match expr {
        Expr::Sequence(mut items) if items.len() == 1 => items.pop().expect("one item exists"),
        other => other,
    }
}

/// Reuse the existing black selector after a raw fallback fragment once visible content already appeared.
fn raw_fallback_reuses_black_selector(items: &[Expr], index: usize) -> bool {
    if index == 0 || !matches!(items[index - 1], Expr::RawTex(_)) {
        return false;
    }
    items[..index - 1].iter().any(|item| {
        !matches!(item, Expr::RawTex(_) | Expr::Space(_))
            && !matches!(item, Expr::Sequence(inner) if inner.is_empty())
    })
}

/// Return true for nodes that begin by selecting their own color.
fn expr_sets_own_color(expr: &Expr) -> bool {
    match expr {
        Expr::DefaultColor(_) => true,
        Expr::Color { .. } | Expr::RawTex(_) => true,
        Expr::Subarray { .. } => true,
        Expr::Style {
            kind: StyleKind::Text,
            content,
        } => match unwrap_single_sequence(content) {
            Expr::Fraction(_, _) => true,
            Expr::Pile {
                kind: PileKind::Parenthesized | PileKind::Binom,
                ..
            } => true,
            Expr::BigOp {
                body: None,
                lower,
                upper,
                ..
            } => lower.is_some() || upper.is_some(),
            Expr::IntegralOp {
                body: None,
                placement: LimitPlacement::NoLimits,
                ..
            } => true,
            _ => false,
        },
        Expr::Pile {
            kind: PileKind::Binom,
            ..
        } => true,
        Expr::HybridLayout(parts) => parts.first().is_some_and(|part| match part {
            HybridPart::Raw(_) => true,
            HybridPart::Line(expr) => expr_sets_own_color(expr),
        }),
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
                ..
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
    let nested_display_style_is_transparent = current_size != SizeState::Full
        && (matches!(content, Expr::Fraction(_, _)) || display_style_is_transparent);
    if kind == StyleKind::Display
        && ((suppress_restore && display_style_is_transparent)
            || nested_display_style_is_transparent)
    {
        // Bug-fix: once a script-sized slot is already open, MathType keeps
        // display-style native templates at that inherited size instead of
        // bouncing back to full size first.
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
    if matches!(kind, StyleKind::Script | StyleKind::ScriptScript) {
        if let Some((width, mut rest)) = leading_space_rest(content) {
            // Bug-fix: raw-prefixed script-style wrappers such as
            // `\rlap{\scriptstyle{\ \ \ \text{shorter}}}` keep leading spacing on
            // the inherited/default path and only switch size once visible glyphs
            // begin inside the styled content.
            let ignore_size_change = writer.parent_sequence_previous_was_raw;
            write_space_without_color(width, out);
            while let [Expr::Space(next_width), tail @ ..] = rest {
                write_space_without_color(*next_width, out);
                rest = tail;
            }
            if rest.is_empty() {
                return Ok(WriteState {
                    size: current_size,
                    color: ColorState::Default,
                });
            }
            if changed_size && !suppress_restore && !ignore_size_change {
                write_size(target_size, out);
            }
            let rest_expr = Expr::Sequence(rest.to_vec());
            let styled_size = if ignore_size_change {
                current_size
            } else {
                target_size
            };
            if ignore_size_change && !expr_starts_with_self_opening(&rest_expr) {
                writer.ensure_black_color_def(out);
                color_black(out);
            }
            let state = write_expr(&rest_expr, out, styled_size, writer)?;
            if changed_size
                && !suppress_restore
                && !ignore_size_change
                && state.size != current_size
            {
                write_size(current_size, out);
            }
            return Ok(WriteState {
                size: current_size,
                color: state.color,
            });
        }
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
    let restore_inside_template =
        !suppress_restore && current_size == SizeState::Full && inner_size != current_size;
    if restore_inside_template {
        // Bug-fix: inside sub/sup slots, MathType leaves the text-style fraction
        // at the inner size and lets the surrounding script separator restore it.
        write_size(current_size, out);
    }
    Ok(WriteState {
        size: if restore_inside_template || suppress_restore {
            current_size
        } else {
            inner_size
        },
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
    if current_size != SizeState::Full {
        // Bug-fix: script-sized text-style piles stay at the inherited slot size
        // instead of shrinking once more before the pile template opens.
        return write_pile(
            PileKind::Parenthesized,
            upper,
            lower,
            out,
            current_size,
            writer,
        );
    }
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
    if current_size != SizeState::Full {
        // Bug-fix: MathType keeps script-sized `\tbinom` payloads at the current
        // script size instead of forcing an extra Sub2 wrapper around the pile.
        return write_binom_pile(upper, lower, out, current_size, writer);
    }
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

/// Write inline/text-style bodyless integrals, which MathType also stores with tmINTOP.
fn write_textstyle_bodyless_integral(
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
    let variation = match (lower.is_some(), upper.is_some()) {
        (true, true) => 0x30,
        (false, true) => 0x20,
        _ => 0x10,
    };
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
    write_named_big_operator_glyph_line("integral", out, writer)?;
    out.push(0x00);
    if !suppress_restore && script_size != current_size {
        write_size(current_size, out);
    }
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
}

/// Write a bodyless integral with explicit limit slots, which reuses tmSUMOP layout.
fn write_bodyless_integral_limits(
    lower: Option<&Expr>,
    upper: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let script_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    let variation = match (lower.is_some(), upper.is_some()) {
        (true, true) => 0x70,
        (false, true) => 0x60,
        _ => 0x50,
    };
    out.extend_from_slice(&[0x03, 0x00, 0x16, variation, 0x00]);
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
    write_named_big_operator_glyph_line("integral", out, writer)?;
    out.push(0x00);
    Ok(WriteState {
        size: script_size,
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

/// Build MathType's text-style side-script form for bodyless `\sum`-style operators.
fn no_limits_big_op_expr(
    kind: BigOpKind,
    lower: Option<&Expr>,
    upper: Option<&Expr>,
    body: Option<&Expr>,
) -> Expr {
    let operator = Expr::Style {
        kind: StyleKind::Text,
        content: Box::new(Expr::BigOp {
            kind,
            lower: lower.cloned().map(Box::new),
            upper: upper.cloned().map(Box::new),
            body: None,
            placement: LimitPlacement::Limits,
        }),
    };
    if let Some(body) = body {
        Expr::Sequence(vec![operator, body.clone()])
    } else {
        operator
    }
}

/// Write one parsed `\lim`/`\sup` node while preserving explicit `\nolimits`.
fn write_limit_expr(
    name: &str,
    lower: Option<&Expr>,
    upper: Option<&Expr>,
    placement: LimitPlacement,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if placement == LimitPlacement::NoLimits {
        let expr = Expr::Script {
            base: Box::new(Expr::FunctionName(name.to_string())),
            sub: lower.cloned().map(Box::new),
            sup: upper.cloned().map(Box::new),
        };
        write_expr(&expr, out, current_size, writer)
    } else {
        write_limit(name, lower, upper, out, current_size, writer)
    }
}

/// Bundle the semantic inputs for one `\mathop` write operation.
struct MathOpWrite<'a> {
    content: &'a Expr,
    lower: Option<&'a Expr>,
    upper: Option<&'a Expr>,
    placement: LimitPlacement,
    leading_space: bool,
}

/// Write one parsed `\mathop` node, preserving limit placement and bare-atom spacing.
fn write_math_op_expr(
    spec: MathOpWrite<'_>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let MathOpWrite {
        content,
        lower,
        upper,
        placement,
        leading_space,
    } = spec;
    if placement == LimitPlacement::NoLimits && (lower.is_some() || upper.is_some()) {
        let expr = Expr::Script {
            base: Box::new(content.clone()),
            sub: lower.cloned().map(Box::new),
            sup: upper.cloned().map(Box::new),
        };
        write_expr(&expr, out, current_size, writer)
    } else if lower.is_some() || upper.is_some() {
        write_custom_limit(content, lower, upper, out, current_size, writer)
    } else if leading_space {
        let expr = Expr::Sequence(vec![
            Expr::Font {
                kind: FontKind::RomanText,
                content: Box::new(Expr::Char(' ')),
            },
            content.clone(),
        ]);
        write_expr(&expr, out, current_size, writer)
    } else {
        write_expr(content, out, current_size, writer)
    }
}

/// Bundle the semantic inputs for one integral write operation.
struct IntegralWrite<'a> {
    kind: IntegralKind,
    body: Option<&'a Expr>,
    lower: Option<&'a Expr>,
    upper: Option<&'a Expr>,
    placement: LimitPlacement,
}

/// Write one parsed integral node while preserving side-script versus limit placement.
fn write_integral_expr(
    spec: IntegralWrite<'_>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let IntegralWrite {
        kind,
        body,
        lower,
        upper,
        placement,
    } = spec;
    if placement == LimitPlacement::NoLimits {
        if body.is_none() {
            let suppress_restore = !writer.force_next_no_limits_integral_style_restore;
            writer.force_next_no_limits_integral_style_restore = false;
            return write_textstyle_bodyless_integral(
                lower,
                upper,
                out,
                current_size,
                suppress_restore,
                writer,
            );
        }
        // Bug-fix: bodyful no-limits integrals, including side-script forms such
        // as `\int_\Omega ...` and contour variants like `\oint_\gamma ...`,
        // still use MathType's native integral template instead of the compact
        // side-script glyph path.
        write_integral_op(kind, body, lower, upper, out, current_size, writer)
    } else if body.is_none() {
        write_bodyless_integral_limits(lower, upper, out, current_size, writer)
    } else {
        write_integral_op(kind, body, lower, upper, out, current_size, writer)
    }
}

/// Write MathType's generic tmLIM template for `\mathop{...}`-style operator bodies.
fn write_custom_limit(
    content: &Expr,
    lower: Option<&Expr>,
    upper: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let variation = match (lower.is_some(), upper.is_some()) {
        (true, false) => 0x10,
        (false, true) => 0x20,
        (true, true) => 0x30,
        (false, false) => return write_expr(content, out, current_size, writer),
    };
    out.extend_from_slice(&[0x03, 0x00, 0x17, variation, 0x00]);
    let main_state = write_line(content, out, current_size, writer)?;
    let limit_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    if main_state.size != limit_size {
        write_size(limit_size, out);
    }
    color_default(out);
    if let Some(lower) = lower {
        let lower_state = write_line(lower, out, limit_size, writer)?;
        if lower_state.size != limit_size {
            write_size(limit_size, out);
        }
        color_default(out);
    } else {
        write_null_line(out);
    }
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
    let variation = integral_variation(kind, lower.is_some() || upper.is_some());
    out.extend_from_slice(&[0x03, 0x00, 0x0f, variation, 0x00]);
    let bodyless = body.is_none();
    let body_state = if let Some(body) = body {
        color_default(out);
        write_line(body, out, current_size, writer)?
    } else {
        // Bug-fix: MathType still emits a native integral template with an empty
        // body slot when limits are attached to a standalone integral.
        write_null_line(out);
        WriteState {
            size: current_size,
            color: ColorState::Default,
        }
    };
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
    } else if !bodyless && body_state.color != ColorState::Black {
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
            suppress_next_pile_black_selector: false,
            suppress_next_stackrel_color_default: false,
            emit_top_fenced_matrix_color: false,
            suppress_next_style_restore: false,
            force_next_no_limits_integral_style_restore: false,
            suppress_next_limit_restore: false,
            emit_top_color_selector_one: false,
            next_color_selector: 1,
            named_color_selectors: HashMap::new(),
            pending_raw_follow_selector: None,
            top_sequence_starts_default: false,
            fallback_environment_active: false,
            suppress_next_line_black: false,
            line_starts_default: false,
            parent_sequence_has_previous_sibling: false,
            parent_sequence_previous_was_raw: false,
            suppress_next_marked_char_marker: false,
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
    let opened_sans_serif_group = kind == FontKind::MathSf
        && !writer.sans_serif_group_active
        // Bug-fix: raw-prefixed wrappers such as `\color{blue}{\sf y}` keep the
        // visible `\sf` bytes ahead of Arial activation, so defer the font group
        // until the first real sans-serif glyph is emitted.
        && !expr_starts_with_raw_tex(expr);
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
                // Bug-fix: raw-prefixed font wrappers such as `\mathcal{\vb{B}}`
                // inherit MathType's default-color path until the first visible
                // font glyph appears, rather than emitting an immediate black->default
                // transition inside the font-specific writer branch.
                color: if expr_starts_with_raw_tex(expr) {
                    ColorState::Default
                } else {
                    ColorState::Black
                },
            };
            for item in items {
                if state.size != current_size {
                    write_size(current_size, out);
                    state.size = current_size;
                }
                if state.color == ColorState::Black && expr_starts_with_raw_tex(item) {
                    // Bug-fix: font-scoped visible text like `\textbf{Argument #1:}`
                    // still drops back to MathType's default raw-fallback color for
                    // the literal `#` fragment before resuming the font glyph run.
                    color_default(out);
                    state.color = ColorState::Default;
                } else if state.color != ColorState::Black && !expr_starts_with_raw_tex(item) {
                    if font_item_needs_explicit_font_def_before_black(kind, item) {
                        emit_font_defs_for_item(kind, item, out, writer);
                        writer.ensure_black_color_def(out);
                        color_black(out);
                        state.color = ColorState::Black;
                    } else if !font_item_opens_its_own_font_path(kind, item) {
                        // Bug-fix: raw-prefixed font wrappers only need an explicit
                        // black restore when the next visible token does not open its
                        // own font-definition path.
                        color_black(out);
                        state.color = ColorState::Black;
                    }
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
        Expr::ArrowAccent {
            kind: arrow_kind,
            under: false,
            content,
        } if kind == FontKind::RomanText && single_accent_char(content).is_some() => {
            let ch = single_accent_char(content).expect("guard checked single accent char");
            let embellishment = match arrow_kind {
                ArrowAccentKind::Right => 0x0b,
                ArrowAccentKind::Left => 0x0c,
                ArrowAccentKind::LeftRight => 0x0d,
            };
            write_font_embellished_char(kind, ch, embellishment, out, writer)?;
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
        Expr::Fraction(numerator, denominator) => {
            // Bug-fix: font wrappers such as `\boldsymbol{\frac{a}{b}}` push the
            // font into each visible fraction slot in MathType instead of leaving
            // the template operands on the default font path.
            let fraction = Expr::Fraction(
                Box::new(font_wrapped_expr(kind, numerator.as_ref())),
                Box::new(font_wrapped_expr(kind, denominator.as_ref())),
            );
            write_expr(&fraction, out, current_size, writer)
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
            if roman_text_keeps_math_font(ch) {
                // Bug-fix: MathType keeps relation punctuation such as `>` on the
                // native math-symbol path even inside `\mathrm{...}`.
                return write_char(ch, out, writer);
            }
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

/// Return true when the next font-scoped item opens its own font-definition bytes.
fn font_item_opens_its_own_font_path(kind: FontKind, expr: &Expr) -> bool {
    if expr_starts_with_line_font_def(expr)
        || expr_starts_with_euclid_math_one(expr)
        || expr_starts_with_euclid_math_two(expr)
        || expr_starts_with_euclid_fraktur(expr)
    {
        return true;
    }
    let Some(ch) = first_plain_char(expr) else {
        return false;
    };
    match kind {
        FontKind::MathCal | FontKind::MathScr => encoding::mathcal_char(ch)
            .is_ok_and(|entry| entry.font_pos.is_some() && entry.typeface == EXPLICIT_FONT_NEG_1),
        FontKind::MathBb => encoding::mathbb_char(ch)
            .is_ok_and(|entry| entry.font_pos.is_some() && entry.typeface == EXPLICIT_FONT_NEG_1),
        FontKind::MathFrak => encoding::mathfrak_char(ch)
            .is_ok_and(|entry| entry.font_pos.is_some() && entry.typeface == EXPLICIT_FONT_NEG_1),
        _ => false,
    }
}

/// Return true when one raw-prefixed font item needs explicit font defs before black restore.
fn font_item_needs_explicit_font_def_before_black(kind: FontKind, expr: &Expr) -> bool {
    let Some(ch) = first_plain_char(expr) else {
        return false;
    };
    match kind {
        FontKind::MathCal | FontKind::MathScr => encoding::mathcal_char(ch)
            .is_ok_and(|entry| entry.font_pos.is_some() && entry.typeface == EXPLICIT_FONT_NEG_1),
        FontKind::MathBb => encoding::mathbb_char(ch)
            .is_ok_and(|entry| entry.font_pos.is_some() && entry.typeface == EXPLICIT_FONT_NEG_1),
        FontKind::MathFrak => encoding::mathfrak_char(ch)
            .is_ok_and(|entry| entry.font_pos.is_some() && entry.typeface == EXPLICIT_FONT_NEG_1),
        _ => false,
    }
}

/// Emit the explicit font definitions required by the next raw-prefixed font item.
fn emit_font_defs_for_item(
    kind: FontKind,
    expr: &Expr,
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) {
    if !font_item_needs_explicit_font_def_before_black(kind, expr) {
        return;
    }
    match kind {
        FontKind::MathCal | FontKind::MathScr => writer.ensure_euclid_math_one(out),
        FontKind::MathBb => writer.ensure_euclid_math_two(out),
        FontKind::MathFrak => writer.ensure_euclid_fraktur(out),
        _ => {}
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

/// Return true when `\mathrm` should preserve MathType's native math glyph font.
fn roman_text_keeps_math_font(ch: char) -> bool {
    matches!(ch, '<' | '>')
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
    if expr_is_empty_sequence(expr) {
        // Bug-fix: empty template slots such as `\sin \left(\right)` still use
        // a real empty LINE record, but MathType does not open a black run inside it.
        out.push(0x00);
        return Ok(WriteState {
            size: current_size,
            color: ColorState::Default,
        });
    }
    if let Some((width, rest)) = leading_space_rest(expr) {
        write_space_without_color(width, out);
        let mut rest = rest;
        while let [Expr::Space(width), tail @ ..] = rest {
            write_space_without_color(*width, out);
            rest = tail;
        }
        if !rest.is_empty() {
            color_black(out);
            if writer.next_color_selector == 1 {
                writer.next_color_selector = 2;
            }
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
    } else if expr_starts_with_marked_char(expr) {
        out.push(0x0d);
        writer.suppress_next_marked_char_marker = true;
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
        if writer.next_color_selector == 1 {
            writer.next_color_selector = 2;
        }
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
