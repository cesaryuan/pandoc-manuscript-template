use crate::ast::*;
use crate::generated::char_tables::ExplicitFont;
use crate::typeface::{
    EXPLICIT_FONT_NEG_1, EXPLICIT_FONT_NEG_2, FN_EXPAND, FN_FUNCTION, FN_MT_EXTRA, FN_NUMBER,
    FN_SPACE, FN_SYMBOL, FN_TEXT, FN_USER1, FN_VARIABLE, FN_VECTOR,
};

#[path = "mtef/encoding.rs"]
mod encoding;
#[path = "mtef/records.rs"]
mod records;

use records::{
    color_black, color_default, write_expanding_glyph, write_null_line, write_size,
    write_table_char, write_u16, write_unsigned,
};

const MTEF_FIXED_DEFS: &[u8] = &[
    0x13, b'W', b'i', b'n', b'A', b'l', b'l', b'B', b'a', b's', b'i', b'c', b'C', b'o', b'd', b'e',
    b'P', b'a', b'g', b'e', b's', 0x00, 0x11, 0x05, b'T', b'i', b'm', b'e', b's', b' ', b'N', b'e',
    b'w', b' ', b'R', b'o', b'm', b'a', b'n', 0x00, 0x11, 0x03, b'S', b'y', b'm', b'b', b'o', b'l',
    0x00, 0x11, 0x05, b'C', b'o', b'u', b'r', b'i', b'e', b'r', b' ', b'N', b'e', b'w', 0x00, 0x11,
    0x04, b'M', b'T', b' ', b'E', b'x', b't', b'r', b'a', 0x00, 0x13, b'W', b'i', b'n', b'A', b'l',
    b'l', b'C', b'o', b'd', b'e', b'P', b'a', b'g', b'e', b's', 0x00, 0x11, 0x06, 0xcb, 0xce, 0xcc,
    0xe5, 0x00, 0x12, 0x00, 0x08, 0x21, 0x2f, 0x45, 0x8f, 0x44, 0x2f, 0x41, 0x50, 0xf4, 0x10, 0x0f,
    0x47, 0x5f, 0x41, 0x50, 0xf2, 0x1f, 0x1e, 0x41, 0x50, 0xf4, 0x15, 0x0f, 0x41, 0x00, 0xf4, 0x45,
    0xf4, 0x25, 0xf4, 0x8f, 0x42, 0x5f, 0x41, 0x00, 0xf4, 0x10, 0x0f, 0x43, 0x5f, 0x41, 0x00, 0xf4,
    0x8f, 0x45, 0xf4, 0x2a, 0x5f, 0x48, 0xf4, 0x8f, 0x41, 0x00, 0xf4, 0x10, 0x0f, 0x40, 0xf4, 0x8f,
    0x41, 0x7f, 0x48, 0xf4, 0x10, 0x0f, 0x41, 0x2a, 0x5f, 0x44, 0x5f, 0x45, 0xf4, 0x5f, 0x45, 0xf4,
    0x5f, 0x41, 0x0f, 0x0c, 0x01, 0x00, 0x01, 0x00, 0x01, 0x02, 0x02, 0x02, 0x02, 0x00, 0x02, 0x00,
    0x01, 0x01, 0x01, 0x00, 0x03, 0x00, 0x01, 0x00, 0x04, 0x00, 0x05, 0x00,
];

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
    euclid_math_one_typeface: u8,
    euclid_math_two_typeface: u8,
    euclid_fraktur_typeface: u8,
    black_color_defined: bool,
    big_symbol_line_marker_pending: bool,
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
}

/// Build the MTEF stream, including MathType's TeX-source future record.
pub(crate) fn write_mtef(source_latex: &str, expr: &Expr) -> Result<Vec<u8>, String> {
    let mut out = vec![0x05, 0x01, 0x00, 0x07, 0x08];
    out.extend_from_slice(b"DSMT7\0");
    out.push(0x01);
    out.push(0x66);

    let mut source = b"TeX Input Language\0".to_vec();
    source.extend_from_slice(source_latex.as_bytes());
    source.push(0x00);
    write_unsigned(source.len(), &mut out)?;
    out.extend_from_slice(&source);

    out.extend_from_slice(MTEF_FIXED_DEFS);
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
        euclid_math_one_typeface: EXPLICIT_FONT_NEG_1,
        euclid_math_two_typeface: EXPLICIT_FONT_NEG_1,
        euclid_fraktur_typeface: EXPLICIT_FONT_NEG_1,
        black_color_defined: false,
        big_symbol_line_marker_pending: false,
    };
    if expr_is_only_spaces(expr) {
        write_only_spaces(expr, out)?;
        out.extend_from_slice(&[0x00, 0x00]);
        return Ok(());
    } else {
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
        } else if expr_starts_with_standalone_big_glyph(expr) {
            out.push(0x0d);
        }
        if !expr_starts_with_top_matrix(expr)
            && !expr_starts_with_raw_tex(expr)
            && !expr_starts_with_explicit_accent_template(expr)
            && !expr_starts_with_sum_operator_script_base(expr)
            && !expr_starts_with_slotless_big_op(expr)
        {
            writer.ensure_black_color_def(out);
            color_black(out);
        }
    }
    write_expr(expr, out, SizeState::Full, &mut writer)?;
    out.extend_from_slice(&[0x00, 0x00]);
    Ok(())
}

/// Return true when MathType starts the equation body with a MATRIX record.
fn expr_starts_with_top_matrix(expr: &Expr) -> bool {
    match expr {
        Expr::Environment {
            kind: EnvironmentKind::Align | EnvironmentKind::Aligned | EnvironmentKind::Split,
            ..
        } => true,
        Expr::Limit { .. } => true,
        Expr::Underset { .. } => true,
        Expr::Matrix { .. } => true,
        Expr::Style { content, .. } => expr_starts_with_top_matrix(content),
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_top_matrix),
        _ => false,
    }
}

/// Return true when MathType starts this environment as raw TeX fallback text.
fn expr_starts_with_environment_fallback(expr: &Expr) -> bool {
    match expr {
        Expr::Environment {
            kind: EnvironmentKind::Aligned | EnvironmentKind::Split,
            ..
        } => true,
        Expr::Style { content, .. } => expr_starts_with_environment_fallback(content),
        Expr::Sequence(items) => items
            .first()
            .is_some_and(expr_starts_with_environment_fallback),
        _ => false,
    }
}

/// Return true when MathType starts a line with raw unsupported TeX text.
fn expr_starts_with_raw_tex(expr: &Expr) -> bool {
    match expr {
        Expr::RawTex(_) => true,
        Expr::Style { content, .. } => expr_starts_with_raw_tex(content),
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_raw_tex),
        Expr::Script { base, .. } => expr_starts_with_raw_tex(base),
        _ => false,
    }
}

/// Return true when a line starts with a big-symbol command carrying scripts.
fn expr_starts_with_big_symbol_script_base(expr: &Expr) -> bool {
    match expr {
        Expr::Script { base, .. } => matches!(base.as_ref(), Expr::BigSymbol(_)),
        Expr::Style { content, .. } => expr_starts_with_big_symbol_script_base(content),
        Expr::Sequence(items) => items
            .first()
            .is_some_and(expr_starts_with_big_symbol_script_base),
        _ => false,
    }
}

/// Return true when MathType lets an explicit accent template choose its own first color.
fn expr_starts_with_explicit_accent_template(expr: &Expr) -> bool {
    match expr {
        Expr::Accent { kind, .. } => matches!(
            kind,
            AccentKind::Acute | AccentKind::Grave | AccentKind::Check
        ),
        Expr::Style { content, .. } => expr_starts_with_explicit_accent_template(content),
        Expr::Sequence(items) => items
            .first()
            .is_some_and(expr_starts_with_explicit_accent_template),
        _ => false,
    }
}

/// Return true when a line/sequence item starts with a tmSUMOP scripted glyph.
fn expr_starts_with_sum_operator_script_base(expr: &Expr) -> bool {
    match expr {
        Expr::Script { base, .. } => matches!(base.as_ref(), Expr::SumOperatorSymbol(_)),
        Expr::Style { content, .. } => expr_starts_with_sum_operator_script_base(content),
        Expr::Sequence(items) => items
            .first()
            .is_some_and(expr_starts_with_sum_operator_script_base),
        _ => false,
    }
}

/// Return true when MathType prefixes a standalone large glyph with a line marker.
fn expr_starts_with_standalone_big_glyph(expr: &Expr) -> bool {
    match expr {
        Expr::BigSymbol(_) | Expr::SumOperatorSymbol(_) => true,
        Expr::Style { content, .. } => expr_starts_with_standalone_big_glyph(content),
        Expr::Sequence(items) => items
            .first()
            .is_some_and(expr_starts_with_standalone_big_glyph),
        _ => false,
    }
}

/// Return true when a slotless BigOp branch must own its marker/color order.
fn expr_starts_with_slotless_big_op(expr: &Expr) -> bool {
    match expr {
        Expr::BigOp {
            lower: None,
            upper: None,
            body: None,
            ..
        } => true,
        Expr::Style { content, .. } => expr_starts_with_slotless_big_op(content),
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_slotless_big_op),
        _ => false,
    }
}

/// Return true when MathType emits Euclid Math One before the first line def.
fn expr_starts_with_euclid_math_one(expr: &Expr) -> bool {
    match expr {
        Expr::Char(ch) | Expr::BigSymbol(ch) | Expr::SumOperatorSymbol(ch) => {
            encoding::special_char(*ch)
                .is_some_and(|entry| entry.explicit_font == Some(ExplicitFont::EuclidMathOne))
        }
        Expr::CommandSymbol { command, ch } => encoding::command_specific_char(command)
            .map(|entry| entry.explicit_font)
            .or_else(|| encoding::special_char(*ch).map(|entry| entry.explicit_font))
            .is_some_and(|font| font == Some(ExplicitFont::EuclidMathOne)),
        Expr::Font {
            kind: FontKind::MathCal | FontKind::MathScr,
            content,
        } => first_plain_char(content).is_some_and(|ch| {
            encoding::mathcal_char(ch).is_ok_and(|entry| {
                entry.font_pos.is_some() && entry.typeface == EXPLICIT_FONT_NEG_1
            })
        }),
        Expr::Font { content, .. } | Expr::Accent { content, .. } | Expr::Style { content, .. } => {
            expr_starts_with_euclid_math_one(content)
        }
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_euclid_math_one),
        Expr::Script { base, .. } => expr_starts_with_euclid_math_one(base),
        _ => false,
    }
}

/// Return true when MathType emits Euclid Math Two before the first line color.
fn expr_starts_with_euclid_math_two(expr: &Expr) -> bool {
    match expr {
        Expr::Char(ch) | Expr::BigSymbol(ch) | Expr::SumOperatorSymbol(ch) => {
            encoding::special_char(*ch)
                .is_some_and(|entry| entry.explicit_font == Some(ExplicitFont::EuclidMathTwo))
        }
        Expr::CommandSymbol { command, ch } => encoding::command_specific_char(command)
            .map(|entry| entry.explicit_font)
            .or_else(|| encoding::special_char(*ch).map(|entry| entry.explicit_font))
            .is_some_and(|font| font == Some(ExplicitFont::EuclidMathTwo)),
        Expr::Font {
            kind: FontKind::MathBb,
            content,
        } => first_plain_char(content).is_some_and(|ch| {
            encoding::mathbb_char(ch).is_ok_and(|entry| {
                entry.font_pos.is_some() && entry.typeface == EXPLICIT_FONT_NEG_1
            })
        }),
        Expr::Font { content, .. } | Expr::Accent { content, .. } | Expr::Style { content, .. } => {
            expr_starts_with_euclid_math_two(content)
        }
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_euclid_math_two),
        Expr::Script { base, .. } => expr_starts_with_euclid_math_two(base),
        _ => false,
    }
}

/// Return true when MathType emits Euclid Fraktur before selecting line color.
fn expr_starts_with_euclid_fraktur(expr: &Expr) -> bool {
    match expr {
        Expr::Font {
            kind: FontKind::MathFrak,
            content,
        } => first_plain_char(content).is_some_and(|ch| {
            encoding::mathfrak_char(ch).is_ok_and(|entry| {
                entry.font_pos.is_some() && entry.typeface == EXPLICIT_FONT_NEG_1
            })
        }),
        Expr::Style { content, .. } => expr_starts_with_euclid_fraktur(content),
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_euclid_fraktur),
        Expr::Script { base, .. } => expr_starts_with_euclid_fraktur(base),
        _ => false,
    }
}

/// Return true for algorithm-indent formulas that contain only spacing commands.
fn expr_is_only_spaces(expr: &Expr) -> bool {
    match expr {
        Expr::Space(_) => true,
        Expr::Style { content, .. } => expr_is_only_spaces(content),
        Expr::Sequence(items) => !items.is_empty() && items.iter().all(expr_is_only_spaces),
        _ => false,
    }
}

/// Write a pure spacing formula without color records, matching MathType output.
fn write_only_spaces(expr: &Expr, out: &mut Vec<u8>) -> Result<(), String> {
    match expr {
        Expr::Space(width) => out.extend_from_slice(&[0x02, 0x00, FN_SPACE, *width, 0xef]),
        Expr::Sequence(items) => {
            for item in items {
                write_only_spaces(item, out)?;
            }
        }
        _ => return Err("internal error: non-space expression in write_only_spaces".to_string()),
    }
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
            let mut state = WriteState {
                size: current_size,
                color: ColorState::Black,
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
                if state.color != ColorState::Black && !expr_sets_own_color(item) {
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
                if index > 0 && state.color == ColorState::Black && expr_starts_with_raw_tex(item) {
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
                state = write_expr(item, out, state.size, writer)?;
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
            write_function_name(name, out)?;
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
        Expr::Boxed(content) => write_boxed(content, out, current_size, writer)?,
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
            write_integral(*kind, out)?;
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
        Expr::Matrix { kind, rows } => {
            write_matrix(*kind, rows, out, current_size, writer)?;
            WriteState {
                size: current_size,
                color: ColorState::Black,
            }
        }
        Expr::Environment { kind, rows } => {
            write_environment(*kind, rows, out, current_size, writer)?;
            WriteState {
                size: current_size,
                color: ColorState::Black,
            }
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
            out.extend_from_slice(&[0x0f, 0x02]);
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
        Expr::Style { content, .. } => expr_sets_own_color(content),
        Expr::Sequence(items) => items.first().is_some_and(expr_sets_own_color),
        _ => false,
    }
}

/// Write one MTEF CHAR record using MathType's simple font/style choices.
fn write_char(ch: char, out: &mut Vec<u8>, writer: &mut MtefWriter) -> Result<(), String> {
    if let Some(special) = encoding::special_char(ch) {
        write_styled_table_char(
            special.typeface,
            special.mtcode,
            special.font_pos,
            special.explicit_font,
            out,
            writer,
        );
        return Ok(());
    }

    let code = ch as u32;
    if code > u16::MAX as u32 {
        return Err(format!(
            "character is outside BMP and not yet supported: {ch}"
        ));
    }

    if let Some(operator) = encoding::operator_char(ch) {
        write_table_char(operator.typeface, operator.mtcode, operator.font_pos, out);
    } else if is_function_char(ch) {
        out.push(0x02);
        out.push(0x00);
        out.push(FN_FUNCTION);
        write_u16(code as u16, out);
    } else if is_math_symbol_char(ch) {
        out.push(0x02);
        out.push(0x00);
        out.push(FN_SYMBOL);
        write_u16(code as u16, out);
    } else {
        out.push(0x02);
        out.push(0x00);
        out.push(if ch.is_ascii_digit() {
            FN_NUMBER
        } else {
            FN_VARIABLE
        });
        write_u16(code as u16, out);
    }
    Ok(())
}

/// Write a source-command-specific CHAR record when Unicode alone is ambiguous.
fn write_command_symbol(
    command: &str,
    ch: char,
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
    write_styled_table_char(
        symbol.typeface,
        symbol.mtcode,
        symbol.font_pos,
        symbol.explicit_font,
        out,
        writer,
    );
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

/// Write MathType's fnSPACE character used for spacing commands.
fn write_space(width: u8, out: &mut Vec<u8>) {
    color_default(out);
    out.extend_from_slice(&[0x02, 0x00, FN_SPACE, width, 0xef]);
}

/// Write a function-name sequence, marking the first character as function start.
fn write_function_name(name: &str, out: &mut Vec<u8>) -> Result<(), String> {
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
    let target_size = style_size(kind);
    if target_size != current_size {
        write_size(target_size, out);
    }
    let state = write_expr(content, out, target_size, writer)?;
    if state.size != current_size {
        write_size(current_size, out);
    }
    Ok(WriteState {
        size: current_size,
        color: state.color,
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

/// Write text-like CHAR records as UTF-16 code units, including surrogate pairs.
fn write_text_code_units(text: &str, options: u8, out: &mut Vec<u8>) -> Result<(), String> {
    for code in text.encode_utf16() {
        out.push(0x02);
        out.push(options);
        out.push(FN_TEXT);
        write_u16(code, out);
    }
    Ok(())
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
    color_default(out);
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
    if current_size != limit_size {
        write_size(current_size, out);
    }
    color_black(out);
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
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
        IntegralKind::Double | IntegralKind::ContourDouble => 2,
        IntegralKind::Triple | IntegralKind::ContourTriple => 3,
    }
}

/// Return true for contour integral variants that add MathType's loop glyph.
fn integral_has_loop(kind: IntegralKind) -> bool {
    matches!(
        kind,
        IntegralKind::Contour | IntegralKind::ContourDouble | IntegralKind::ContourTriple
    )
}

/// Write the integral glyph MathType appends at the end of integral templates.
fn write_integral_glyph(out: &mut Vec<u8>) -> Result<(), String> {
    write_named_big_operator_glyph("integral", out)
}

/// Write a generated glyph used by MathType's big-operator templates.
fn write_named_big_operator_glyph(name: &str, out: &mut Vec<u8>) -> Result<(), String> {
    let glyph = encoding::big_operator_glyph(name)?;
    out.push(0x02);
    out.push(0x04);
    out.push(glyph.typeface);
    write_u16(glyph.mtcode, out);
    out.push(glyph.font_pos);
    Ok(())
}

/// Write a standalone integral glyph when no template body follows.
fn write_integral(kind: IntegralKind, out: &mut Vec<u8>) -> Result<(), String> {
    if integral_has_loop(kind) {
        write_named_big_operator_glyph("contour_loop", out)?;
    }
    for _ in 0..integral_count(kind) {
        write_integral_glyph(out)?;
    }
    Ok(())
}

/// Write MathType's two-row pile, optionally wrapped in a delimiter pair.
fn write_pile(
    kind: PileKind,
    upper: &Expr,
    lower: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    color_default(out);
    if let Some((left, right)) = pile_delimiters(kind) {
        out.extend_from_slice(&[0x03, 0x00, delimiter_selector(left, right)?, 0x03, 0x00]);
    }
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
    if let Some((left, right)) = pile_delimiters(kind) {
        color_default(out);
        write_delimiter_glyph_pair(left, right, out)?;
        out.push(0x00);
    }
    Ok(WriteState {
        size: current_size,
        color: ColorState::Default,
    })
}

/// Return the optional delimiter pair around a two-row pile.
fn pile_delimiters(kind: PileKind) -> Option<(char, char)> {
    match kind {
        PileKind::Plain => None,
        PileKind::Parenthesized => Some(('(', ')')),
        PileKind::Braced => Some(('{', '}')),
        PileKind::Bracketed => Some(('[', ']')),
    }
}

/// Write MathType's overbrace/underbrace template with an optional annotation slot.
fn write_brace_template(
    kind: BraceKind,
    content: &Expr,
    annotation: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let variation = match kind {
        BraceKind::Over => 0x01,
        BraceKind::Under => 0x00,
    };
    let glyph = match kind {
        BraceKind::Over => 0xfe37,
        BraceKind::Under => 0xfe38,
    };
    write_horizontal_fence_template(
        0x18,
        variation,
        glyph,
        content,
        annotation,
        out,
        current_size,
        writer,
    )
}

/// Write a horizontal brace/bracket HFence template described by MathType's selector table.
fn write_horizontal_fence_template(
    selector: u8,
    variation: u8,
    glyph: u16,
    content: &Expr,
    annotation: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x03, 0x00, selector, variation, 0x00]);
    color_default(out);
    let content_state = write_line(content, out, current_size, writer)?;
    let annotation_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    if annotation.is_some() || content_state.size != annotation_size {
        write_size(annotation_size, out);
    }
    color_default(out);
    let annotation_state = if let Some(annotation) = annotation {
        write_line(annotation, out, annotation_size, writer)?
    } else {
        write_empty_matrix_cell_line(out);
        WriteState {
            size: annotation_size,
            color: ColorState::Default,
        }
    };
    if annotation_state.size != current_size {
        write_size(current_size, out);
    }
    if annotation.is_none() {
        color_black(out);
    }
    write_expanding_glyph(glyph, out);
    out.push(0x00);
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
}

/// Write \stackrel as MathType's above/below stacking template.
fn write_stackrel(
    upper: &Expr,
    lower: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    color_default(out);
    out.extend_from_slice(&[0x03, 0x00, 0x17, 0x20, 0x00]);
    let lower_state = write_line(lower, out, current_size, writer)?;
    let stack_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    if lower_state.size != stack_size {
        write_size(stack_size, out);
    }
    color_default(out);
    write_null_line(out);
    let upper_state = write_line(upper, out, stack_size, writer)?;
    out.push(0x00);
    Ok(WriteState {
        size: upper_state.size,
        color: upper_state.color,
    })
}

/// Write \underset using MathType's tmLIM lower-slot template.
fn write_underset(
    lower: &Expr,
    base: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x03, 0x00, 0x17, 0x10, 0x00]);
    let base_state = write_line(base, out, current_size, writer)?;
    let stack_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    if base_state.size != stack_size {
        write_size(stack_size, out);
    }
    color_default(out);
    let lower_state = write_line(lower, out, stack_size, writer)?;
    if lower_state.size != stack_size {
        write_size(stack_size, out);
    }
    color_default(out);
    write_null_line(out);
    out.push(0x00);
    Ok(WriteState {
        size: stack_size,
        color: ColorState::Default,
    })
}

/// Write MathType's extensible arrow template with an upper label slot.
fn write_xarrow(
    kind: XArrowKind,
    label: &Expr,
    under: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let direction_bit = xarrow_direction_bit(kind);
    let variation = direction_bit | 0x04 | u8::from(under.is_some()) * 0x08;
    out.extend_from_slice(&[0x03, 0x00, 0x0e, variation, 0x00]);
    let label_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    write_size(label_size, out);
    color_default(out);
    let label_state = write_line(label, out, label_size, writer)?;
    if label_state.size != label_size {
        write_size(label_size, out);
    }
    if let Some(under) = under {
        let under_state = write_line(under, out, label_size, writer)?;
        if under_state.size != label_size {
            write_size(label_size, out);
        }
    } else {
        write_null_line(out);
    }
    if current_size != label_size {
        write_size(current_size, out);
    }
    write_expanding_glyph(xarrow_glyph(kind), out);
    out.push(0x00);
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
}

/// Return MathType's left/right variation bit for an extensible arrow command.
fn xarrow_direction_bit(kind: XArrowKind) -> u8 {
    match kind {
        XArrowKind::Left
        | XArrowKind::DoubleLeft
        | XArrowKind::HookLeft
        | XArrowKind::TwoHeadLeft => 0x10,
        XArrowKind::Right
        | XArrowKind::DoubleRight
        | XArrowKind::HookRight
        | XArrowKind::TwoHeadRight
        | XArrowKind::Mapsto
        | XArrowKind::LongEqual
        | XArrowKind::ToFrom => 0x20,
    }
}

/// Return the expandable glyph code used for one x-arrow variant.
fn xarrow_glyph(kind: XArrowKind) -> u16 {
    match kind {
        // MathType uses the template variation to flip ordinary left arrows.
        XArrowKind::Left | XArrowKind::Right => 0x2192,
        XArrowKind::DoubleLeft | XArrowKind::DoubleRight => 0x21d2,
        XArrowKind::HookLeft | XArrowKind::HookRight => 0x21aa,
        XArrowKind::TwoHeadLeft | XArrowKind::TwoHeadRight => 0x21a0,
        XArrowKind::Mapsto => 0x21a6,
        XArrowKind::LongEqual => 0x003d,
        XArrowKind::ToFrom => 0x21c4,
    }
}

/// Write a matrix environment as a real MTEF MATRIX wrapped in its fence template.
fn write_matrix(
    kind: MatrixKind,
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    let Some((left, right, selector)) = (match kind {
        MatrixKind::Plain => None,
        MatrixKind::Parenthesized => Some(('(', ')', 0x01)),
        MatrixKind::Bracketed => Some(('[', ']', 0x03)),
        MatrixKind::Braced => Some(('{', '}', 0x02)),
        MatrixKind::Barred => Some(('|', '|', 0x04)),
        MatrixKind::DoubleBarred => Some(('‖', '‖', 0x05)),
    }) else {
        write_plain_matrix_record(rows, out, current_size, writer)?;
        return Ok(());
    };
    write_fenced_matrix(selector, left, right, rows, out, current_size, writer)?;
    Ok(())
}

/// Write alignment-like environments using MTEF layout records, not fixture bytes.
fn write_environment(
    kind: EnvironmentKind,
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    match kind {
        EnvironmentKind::Align | EnvironmentKind::AlignAt => {
            write_align_matrix_record(rows, out, current_size, writer)
        }
        EnvironmentKind::Split => {
            write_environment_fallback("split", "&", "\\end", rows, out, current_size, writer)
        }
        EnvironmentKind::Aligned => {
            write_environment_fallback("aligned", "nn&", "nn\\end", rows, out, current_size, writer)
        }
        EnvironmentKind::AlignedAt => write_align_matrix_record(rows, out, current_size, writer),
        EnvironmentKind::Gather | EnvironmentKind::Gathered => {
            write_matrix_record(rows, out, current_size, writer)
        }
        EnvironmentKind::Cases => write_left_fenced_matrix(rows, out, current_size, writer),
        EnvironmentKind::RightCases => write_right_fenced_matrix(rows, out, current_size, writer),
    }
}

/// Emulate MathType TeX Input's fallback for unsupported alignment environments.
fn write_environment_fallback(
    name: &str,
    separator: &str,
    end_command: &str,
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    write_raw_tex_text("\\begin", out)?;
    writer.ensure_black_color_def(out);
    color_black(out);
    for ch in name.chars() {
        write_char(ch, out, writer)?;
    }
    let mut state = WriteState {
        size: current_size,
        color: ColorState::Black,
    };
    for (row_index, row) in rows.iter().enumerate() {
        for (cell_index, cell) in row.iter().enumerate() {
            if row_index > 0 && cell_index == 0 && expr_is_empty_sequence(cell) {
                continue;
            }
            let mut after_raw_separator = false;
            if cell_index > 0 {
                if state.size != current_size {
                    write_size(current_size, out);
                    state.size = current_size;
                }
                if state.color != ColorState::Default {
                    color_default(out);
                }
                write_raw_tex_text(separator, out)?;
                after_raw_separator = true;
                if !expr_starts_with_space(cell) {
                    color_black(out);
                }
            }
            if !after_raw_separator
                && row_index > 0
                && state.color != ColorState::Default
                && expr_starts_with_environment_fallback(cell)
            {
                color_default(out);
                state.color = ColorState::Default;
            }
            state = if name == "split" && !after_raw_separator {
                if let Some(nested_state) =
                    write_nested_aligned_fallback(cell, out, current_size, writer)?
                {
                    nested_state
                } else if expr_starts_with_space(cell) {
                    write_fallback_cell_after_default_space(cell, out, current_size, writer)?
                } else {
                    write_expr(cell, out, current_size, writer)?
                }
            } else if after_raw_separator && expr_starts_with_space(cell) {
                write_fallback_cell_after_default_space(cell, out, current_size, writer)?
            } else {
                write_expr(cell, out, current_size, writer)?
            };
        }
    }
    if state.size != current_size {
        write_size(current_size, out);
    }
    color_default(out);
    write_raw_tex_text(end_command, out)?;
    color_black(out);
    for ch in name.chars() {
        write_char(ch, out, writer)?;
    }
    Ok(())
}

/// Write nested aligned environments inside split using MathType's split fallback separators.
fn write_nested_aligned_fallback(
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<Option<WriteState>, String> {
    let Expr::Sequence(items) = expr else {
        return Ok(None);
    };
    let [Expr::Environment {
        kind: EnvironmentKind::Aligned,
        rows,
    }] = items.as_slice()
    else {
        return Ok(None);
    };
    write_environment_fallback("aligned", "&", "\\end", rows, out, current_size, writer)?;
    Ok(Some(WriteState {
        size: current_size,
        color: ColorState::Black,
    }))
}

/// Return true when a fallback cell starts with TeX spacing such as \quad.
fn expr_starts_with_space(expr: &Expr) -> bool {
    match expr {
        Expr::Space(_) => true,
        Expr::Style { content, .. } => expr_starts_with_space(content),
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_space),
        _ => false,
    }
}

/// Write a fallback aligned cell after raw separators already selected default color.
fn write_fallback_cell_after_default_space(
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if let Some((width, rest)) = leading_space_rest(expr) {
        write_space_without_color(width, out);
        if rest.is_empty() {
            return Ok(WriteState {
                size: current_size,
                color: ColorState::Default,
            });
        }
        color_black(out);
        let rest_expr = Expr::Sequence(rest.to_vec());
        return write_expr(&rest_expr, out, current_size, writer);
    }
    write_expr(expr, out, current_size, writer)
}

/// Write align rows, padding omitted leading alignment cells on continuation rows.
fn write_align_matrix_record(
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    let col_count = rows.iter().map(Vec::len).max().unwrap_or(0);
    let padded = rows
        .iter()
        .map(|row| {
            if row.len() + 1 == col_count {
                let mut padded_row = Vec::with_capacity(col_count);
                padded_row.push(Expr::Sequence(Vec::new()));
                padded_row.extend(row.iter().cloned());
                padded_row
            } else {
                row.clone()
            }
        })
        .collect::<Vec<_>>();
    write_matrix_record(&padded, out, current_size, writer)
}

/// Write a two-sided fence template whose main slot is a MATRIX record.
fn write_fenced_matrix(
    selector: u8,
    left: char,
    right: char,
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    out.extend_from_slice(&[0x03, 0x00, selector, 0x03, 0x00]);
    color_default(out);
    write_matrix_slot_line(rows, out, current_size, writer, MatrixHeaderStyle::Fenced)?;
    write_delimiter_glyph(left, out)?;
    write_delimiter_glyph(right, out)?;
    out.push(0x00);
    Ok(())
}

/// Write \begin{cases} as MathType's left-brace fence around a MATRIX.
fn write_left_fenced_matrix(
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    out.extend_from_slice(&[0x03, 0x00, 0x02, 0x01, 0x00]);
    color_default(out);
    write_matrix_slot_line(rows, out, current_size, writer, MatrixHeaderStyle::Cases)?;
    write_delimiter_glyph('{', out)?;
    out.push(0x00);
    Ok(())
}

/// Write right-braced cases as a MATRIX inside a right-only fence template.
fn write_right_fenced_matrix(
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    out.extend_from_slice(&[0x03, 0x00, 0x02, 0x02, 0x00]);
    color_default(out);
    write_matrix_slot_line(rows, out, current_size, writer, MatrixHeaderStyle::Cases)?;
    write_delimiter_glyph('}', out)?;
    out.push(0x00);
    Ok(())
}

/// Write a template slot line that contains only one MATRIX object.
fn write_matrix_slot_line(
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
    header_style: MatrixHeaderStyle,
) -> Result<(), String> {
    out.extend_from_slice(&[0x01, 0x00]);
    let matrix_state =
        write_matrix_record_with_header(rows, out, current_size, writer, header_style)?;
    out.push(0x00);
    if header_style == MatrixHeaderStyle::Fenced && matrix_state.size != current_size {
        write_size(current_size, out);
        color_black(out);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MatrixHeaderStyle {
    Cases,
    Plain,
    Fenced,
}

/// Write MathType's MATRIX record and one LINE object for each cell.
fn write_matrix_record(
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    write_matrix_record_with_header(rows, out, current_size, writer, MatrixHeaderStyle::Cases)?;
    Ok(())
}

/// Write standalone matrix/array environments using MathType's centered columns.
fn write_plain_matrix_record(
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    write_matrix_record_with_header(rows, out, current_size, writer, MatrixHeaderStyle::Plain)?;
    Ok(())
}

/// Write a MATRIX record; fenced matrices use MathType's centered column style byte.
fn write_matrix_record_with_header(
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
    header_style: MatrixHeaderStyle,
) -> Result<WriteState, String> {
    let row_count = u8::try_from(rows.len()).map_err(|_| "matrix has too many rows".to_string())?;
    let col_count = rows.iter().map(Vec::len).max().unwrap_or(0);
    let col_count =
        u8::try_from(col_count).map_err(|_| "matrix has too many columns".to_string())?;

    let column_style = match header_style {
        MatrixHeaderStyle::Cases => 0x00,
        MatrixHeaderStyle::Plain => 0x01,
        MatrixHeaderStyle::Fenced => 0x01,
    };
    out.extend_from_slice(&[0x05, 0x00, 0x01, column_style, 0x01, row_count, col_count]);
    out.extend(std::iter::repeat(0x00).take(partition_byte_count(row_count)));
    out.extend(std::iter::repeat(0x00).take(partition_byte_count(col_count)));
    let mut cell_ordinal = 0usize;
    let mut previous_cell_was_empty = false;
    let total_cells = rows.len() * col_count as usize;
    let mut final_cell_state = WriteState {
        size: current_size,
        color: ColorState::Default,
    };
    for row in rows {
        for col_index in 0..col_count as usize {
            if header_style != MatrixHeaderStyle::Fenced
                && cell_ordinal > 0
                && !previous_cell_was_empty
            {
                color_default(out);
            }
            let is_last_cell = cell_ordinal + 1 == total_cells;
            if let Some(cell) = row.get(col_index) {
                previous_cell_was_empty = expr_is_empty_sequence(cell);
                final_cell_state = write_matrix_cell_line(cell, out, current_size, writer)?;
            } else {
                previous_cell_was_empty = true;
                write_empty_matrix_cell_line(out);
                final_cell_state = WriteState {
                    size: current_size,
                    color: ColorState::Default,
                };
            }
            if final_cell_state.size != current_size
                && !(header_style == MatrixHeaderStyle::Fenced && is_last_cell)
            {
                write_size(current_size, out);
            } else if header_style == MatrixHeaderStyle::Fenced
                && !is_last_cell
                && !previous_cell_was_empty
            {
                color_default(out);
            }
            cell_ordinal += 1;
        }
    }
    out.push(0x00);
    Ok(final_cell_state)
}

/// Return the packed two-bit partition-byte count for MATRIX dividers.
fn partition_byte_count(count: u8) -> usize {
    (count as usize + 4) / 4
}

/// Write one MATRIX cell line, preserving MathType's empty-cell form.
fn write_matrix_cell_line(
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if expr_is_empty_sequence(expr) {
        write_empty_matrix_cell_line(out);
        return Ok(WriteState {
            size: current_size,
            color: ColorState::Default,
        });
    }
    write_line(expr, out, current_size, writer)
}

/// Write the non-null but object-empty LINE MathType uses for empty cells.
fn write_empty_matrix_cell_line(out: &mut Vec<u8>) {
    out.extend_from_slice(&[0x01, 0x00, 0x00]);
}

/// Return true for parser-produced empty cells in alignment environments.
fn expr_is_empty_sequence(expr: &Expr) -> bool {
    matches!(expr, Expr::Sequence(items) if items.is_empty())
}

/// Write a font-scoped expression for the MathType font commands used here.
fn write_font_expr(
    kind: FontKind,
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    match expr {
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
        other => write_expr(other, out, current_size, writer),
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
            out.push(0x02);
            out.push(0x00);
            out.push(FN_VECTOR);
            write_u16(code as u16, out);
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
            out.extend_from_slice(&[
                0x11, 0x05, b'A', b'r', b'i', b'a', b'l', 0x00, 0x08, 0x06, 0x00,
            ]);
            color_black(out);
            out.push(0x02);
            out.push(0x00);
            out.push(EXPLICIT_FONT_NEG_1);
            write_u16(code as u16, out);
        }
        FontKind::MathTt => {
            write_table_char(FN_USER1, code as u16, None, out);
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

/// Write simple MathType embellishments such as \bar{I} and \hat{P}.
fn write_accent_expr(
    kind: AccentKind,
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if matches!(
        kind,
        AccentKind::Acute | AccentKind::Grave | AccentKind::Check
    ) {
        return write_explicit_accent_template(kind, expr, out, current_size, writer);
    }
    if let Some((font_kind, ch)) = single_font_char(expr) {
        match (kind, font_kind) {
            (
                AccentKind::Bar
                | AccentKind::Hat
                | AccentKind::Breve
                | AccentKind::Dot
                | AccentKind::Ddot
                | AccentKind::Dddot
                | AccentKind::Ddddot
                | AccentKind::Tilde
                | AccentKind::UnderTilde,
                None,
            ) => {
                write_embellished_char(ch, &[kind], out)?;
                return Ok(WriteState {
                    size: current_size,
                    color: ColorState::Black,
                });
            }
            (AccentKind::Hat, Some(FontKind::Bold)) => {
                write_hat_template_for_bold_char(ch, out)?;
                return Ok(WriteState {
                    size: current_size,
                    color: ColorState::Black,
                });
            }
            _ => {}
        }
    }
    if kind == AccentKind::UnderTilde {
        return write_under_tilde_run(expr, out, current_size);
    }
    if let Some(ch) = widehat_bar_char(expr) {
        write_embellished_char(ch, &[AccentKind::Bar, AccentKind::Hat], out)?;
        return Ok(WriteState {
            size: current_size,
            color: ColorState::Black,
        });
    }
    if kind == AccentKind::Hat {
        return write_hat_template(expr, out, current_size, writer);
    }
    write_expr(expr, out, current_size, writer)
}

/// Write simple multi-character \utilde content with one documented embU_TILDE per char.
fn write_under_tilde_run(
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
) -> Result<WriteState, String> {
    let chars = simple_char_run(expr)
        .ok_or_else(|| "\\utilde currently supports only simple character runs".to_string())?;
    for ch in chars {
        write_embellished_char(ch, &[AccentKind::UnderTilde], out)?;
    }
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
}

/// Extract a parser-vetted run of plain characters for per-character embellishments.
fn simple_char_run(expr: &Expr) -> Option<Vec<char>> {
    match expr {
        Expr::Char(ch) => Some(vec![*ch]),
        Expr::Sequence(items) if !items.is_empty() => {
            let mut chars = Vec::new();
            for item in items {
                chars.extend(simple_char_run(item)?);
            }
            Some(chars)
        }
        _ => None,
    }
}

/// Write MathType's vector-arrow template for arrow accents over/under content.
fn write_arrow_accent_template(
    kind: ArrowAccentKind,
    under: bool,
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if kind == ArrowAccentKind::Right && !under {
        if let Some((None, ch)) = single_font_char(expr) {
            write_embellished_char_with_code(ch, 0x0b, out)?;
            return Ok(WriteState {
                size: current_size,
                color: ColorState::Black,
            });
        }
    }
    out.extend_from_slice(&[0x03, 0x00, 0x1f, arrow_accent_variation(kind, under), 0x00]);
    color_default(out);
    let line_state = write_line(expr, out, current_size, writer)?;
    if line_state.size != current_size {
        write_size(current_size, out);
    }
    if line_state.color != ColorState::Black {
        color_black(out);
    }
    write_delimiter_glyph(arrow_accent_glyph(kind, under), out)?;
    out.push(0x00);
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
}

/// Return MathType's tmVEC variation bits for an arrow accent.
fn arrow_accent_variation(kind: ArrowAccentKind, under: bool) -> u8 {
    let direction = match kind {
        ArrowAccentKind::Left | ArrowAccentKind::LeftHarpoon => 0x01,
        ArrowAccentKind::Right | ArrowAccentKind::RightHarpoon => 0x02,
        ArrowAccentKind::LeftRight => 0x03,
    };
    let harpoon = if matches!(
        kind,
        ArrowAccentKind::LeftHarpoon | ArrowAccentKind::RightHarpoon
    ) {
        0x08
    } else {
        0x00
    };
    direction | harpoon | u8::from(under) * 0x04
}

/// Return the expanding combining glyph MathType appends after a tmVEC slot.
fn arrow_accent_glyph(kind: ArrowAccentKind, under: bool) -> char {
    match (kind, under) {
        (ArrowAccentKind::Left, false) => '\u{20d6}',
        (ArrowAccentKind::Right, false) => '\u{20d7}',
        (ArrowAccentKind::LeftRight, false) => '\u{20e1}',
        (ArrowAccentKind::Left, true) => '\u{20ee}',
        (ArrowAccentKind::Right, true) => '\u{20ef}',
        (ArrowAccentKind::LeftRight, true) => '\u{034d}',
        (ArrowAccentKind::LeftHarpoon, false) => '\u{20d0}',
        (ArrowAccentKind::RightHarpoon, false) => '\u{20d1}',
        (ArrowAccentKind::LeftHarpoon | ArrowAccentKind::RightHarpoon, true) => {
            unreachable!("under harpoon accents are not parsed")
        }
    }
}

/// Write long overline/underline templates for multi-character content.
fn write_bar_template(
    kind: BarTemplateKind,
    content: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let selector = match kind {
        BarTemplateKind::Under => 0x0c,
        BarTemplateKind::Over => 0x0d,
    };
    out.extend_from_slice(&[0x03, 0x00, selector, 0x00, 0x00]);
    color_default(out);
    let content_state = write_line(content, out, current_size, writer)?;
    out.push(0x00);
    Ok(content_state)
}

/// Extract a single character, preserving simple font wrapper information.
fn single_font_char(expr: &Expr) -> Option<(Option<FontKind>, char)> {
    match expr {
        Expr::Char(ch) => Some((None, *ch)),
        Expr::Sequence(items) if items.len() == 1 => single_font_char(&items[0]),
        Expr::Font { kind, content } => single_font_char(content).map(|(_, ch)| (Some(*kind), ch)),
        _ => None,
    }
}

/// Return the first plain character inside a font command before emitting color.
fn first_plain_char(expr: &Expr) -> Option<char> {
    match expr {
        Expr::Char(ch) => Some(*ch),
        Expr::Sequence(items) => items.first().and_then(first_plain_char),
        Expr::Style { content, .. } => first_plain_char(content),
        _ => None,
    }
}

/// Extract the single character from \widehat{\bar{x}} so both accents share one CHAR record.
fn widehat_bar_char(expr: &Expr) -> Option<char> {
    match expr {
        Expr::Accent {
            kind: AccentKind::Bar,
            content,
        } => single_font_char(content).and_then(|(font, ch)| font.is_none().then_some(ch)),
        Expr::Sequence(items) if items.len() == 1 => widehat_bar_char(&items[0]),
        _ => None,
    }
}

/// Write MathType's hat template form used for hats over bold characters.
fn write_hat_template_for_bold_char(ch: char, out: &mut Vec<u8>) -> Result<(), String> {
    out.extend_from_slice(&[0x03, 0x00, 0x21, 0x00, 0x00]);
    color_default(out);
    out.extend_from_slice(&[0x01, 0x00]);
    color_black(out);
    write_font_char(
        FontKind::Bold,
        ch,
        out,
        &mut MtefWriter {
            euclid_math_one_defined: true,
            euclid_math_two_defined: true,
            euclid_fraktur_defined: false,
            euclid_math_one_typeface: EXPLICIT_FONT_NEG_1,
            euclid_math_two_typeface: EXPLICIT_FONT_NEG_2,
            euclid_fraktur_typeface: EXPLICIT_FONT_NEG_1,
            black_color_defined: true,
            big_symbol_line_marker_pending: false,
        },
    )?;
    out.push(0x00);
    out.extend_from_slice(&[0x02, 0x00, FN_EXPAND, 0x02, 0x03, 0x00]);
    Ok(())
}

/// Write MathType's general hat template for non-embellished content.
fn write_hat_template(
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x03, 0x00, 0x21, 0x00, 0x00]);
    color_default(out);
    let line_state = write_line(expr, out, current_size, writer)?;
    if line_state.size != current_size {
        write_size(current_size, out);
    }
    if line_state.color != ColorState::Black {
        color_black(out);
    }
    out.extend_from_slice(&[0x02, 0x00, FN_EXPAND, 0x02, 0x03]);
    out.push(0x00);
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
}

/// Return MathType's EMBELL subtype for single-character accent commands.
fn embellishment_code(kind: AccentKind) -> u8 {
    match kind {
        AccentKind::Dot => 0x02,
        AccentKind::Ddot => 0x03,
        AccentKind::Dddot => 0x04,
        AccentKind::Tilde => 0x08,
        AccentKind::Hat | AccentKind::WideHat => 0x09,
        AccentKind::Bar => 0x11,
        AccentKind::Breve => 0x14,
        AccentKind::Ddddot => 0x18,
        AccentKind::UnderTilde => 0x1e,
        AccentKind::Acute | AccentKind::Grave | AccentKind::Check => {
            unreachable!("explicit accent templates do not use EMBELL records")
        }
    }
}

/// Return the visible accent glyph MathType places in the upper template slot.
fn explicit_accent_mtcode(kind: AccentKind) -> u16 {
    match kind {
        AccentKind::Acute => 0x00b4,
        AccentKind::Grave => 0x0060,
        AccentKind::Check => 0x02c7,
        _ => unreachable!("not an explicit accent template"),
    }
}

/// Write MathType's explicit accent template used by \acute, \grave, and \check.
fn write_explicit_accent_template(
    kind: AccentKind,
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x03, 0x00, 0x17, 0x20, 0x00]);
    let lower_state = write_line(expr, out, current_size, writer)?;
    let stack_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    if lower_state.size != stack_size {
        write_size(stack_size, out);
    }
    color_default(out);
    write_null_line(out);
    write_explicit_accent_line(kind, out);
    out.push(0x00);
    Ok(WriteState {
        size: stack_size,
        color: ColorState::Black,
    })
}

/// Write the upper glyph line in MathType's explicit accent template.
fn write_explicit_accent_line(kind: AccentKind, out: &mut Vec<u8>) {
    out.extend_from_slice(&[0x01, 0x00]);
    color_black(out);
    write_table_char(FN_FUNCTION, explicit_accent_mtcode(kind), None, out);
    out.push(0x00);
}

/// Write a CHAR record with one or more embellishments attached.
fn write_embellished_char(ch: char, kinds: &[AccentKind], out: &mut Vec<u8>) -> Result<(), String> {
    let codes = kinds
        .iter()
        .map(|kind| embellishment_code(*kind))
        .collect::<Vec<_>>();
    write_embellished_char_codes(ch, &codes, out)
}

/// Write a CHAR record with one explicitly probed EMBELL subtype attached.
fn write_embellished_char_with_code(
    ch: char,
    embellishment: u8,
    out: &mut Vec<u8>,
) -> Result<(), String> {
    write_embellished_char_codes(ch, &[embellishment], out)
}

/// Write a CHAR record with one or more raw EMBELL subtype bytes attached.
fn write_embellished_char_codes(
    ch: char,
    embellishments: &[u8],
    out: &mut Vec<u8>,
) -> Result<(), String> {
    let code = ch as u32;
    if code > u16::MAX as u32 {
        return Err(format!("embellished character is outside BMP: {ch}"));
    }
    out.push(0x02);
    out.push(0x01);
    out.push(if ch.is_ascii_digit() {
        FN_NUMBER
    } else {
        FN_VARIABLE
    });
    write_u16(code as u16, out);
    for embellishment in embellishments {
        out.extend_from_slice(&[0x06, 0x00, *embellishment]);
    }
    out.push(0x00);
    Ok(())
}

/// Return true for punctuation MathType writes with the function style.
fn is_function_char(ch: char) -> bool {
    matches!(
        ch,
        '(' | ')' | '[' | ']' | '{' | '}' | '|' | ',' | '.' | ':' | ';' | '/'
    )
}

/// Return true for BMP Unicode blocks that primarily hold math symbols.
fn is_math_symbol_char(ch: char) -> bool {
    matches!(
        ch as u32,
        0x2030..=0x203f
            | 0x2100..=0x214f
            | 0x2190..=0x21ff
            | 0x2200..=0x22ff
            | 0x2300..=0x23ff
            | 0x2571
            | 0x2572
            | 0x25a0..=0x25ff
            | 0x2600..=0x26ff
            | 0x27c0..=0x27ff
            | 0x2900..=0x2aff
    )
}

/// Write a MathType fraction template with numerator and denominator slots.
fn write_fraction(
    numerator: &Expr,
    denominator: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x03, 0x00, 0x0b, 0x00, 0x00]);
    color_default(out);
    let numerator_state = write_line(numerator, out, current_size, writer)?;
    if numerator_state.size != current_size {
        write_size(current_size, out);
        if numerator_state.color != ColorState::Default {
            color_default(out);
        }
    } else {
        color_default(out);
    }
    let denominator_state = write_line(denominator, out, current_size, writer)?;
    out.push(0x00);
    Ok(denominator_state)
}

/// Write a square-root template with a null nth-root index slot.
fn write_sqrt(
    radicand: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x03, 0x00, 0x0a, 0x00, 0x00]);
    color_default(out);
    let radicand_state = write_line(radicand, out, current_size, writer)?;
    if radicand_state.size != SizeState::Sub {
        write_size(SizeState::Sub, out);
    }
    if radicand_state.color != ColorState::Black {
        color_black(out);
    }
    write_null_line(out);
    out.push(0x00);
    Ok(WriteState {
        size: SizeState::Sub,
        color: ColorState::Black,
    })
}

/// Write a square \boxed template with all four sides enabled.
fn write_boxed(
    content: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x03, 0x00, 0x25, 0x1e, 0x00]);
    color_default(out);
    let content_state = write_line(content, out, current_size, writer)?;
    out.push(0x00);
    Ok(content_state)
}

/// Write MathType's overstrike template used by cancel-like commands.
fn write_strike_template(
    kind: StrikeKind,
    content: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let variation = match kind {
        StrikeKind::Horizontal => 0x01,
        StrikeKind::Up => 0x02,
        StrikeKind::Down => 0x04,
        StrikeKind::Both => 0x06,
    };
    out.extend_from_slice(&[0x03, 0x00, 0x24, variation, 0x00]);
    color_default(out);
    let content_state = write_line(content, out, current_size, writer)?;
    out.push(0x00);
    Ok(content_state)
}

/// Write MathType's big-operator template; the following term is its first slot.
fn write_big_op(
    kind: BigOpKind,
    body: Option<&Expr>,
    lower: Option<&Expr>,
    upper: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let Some(body) = body else {
        if lower.is_none() && upper.is_none() {
            out.push(0x0d);
            writer.ensure_black_color_def(out);
            color_black(out);
            write_standalone_big_op_glyph(kind, out)?;
            return Ok(WriteState {
                size: current_size,
                color: ColorState::Black,
            });
        }
        return write_standalone_big_op_limits(kind, lower, upper, out, current_size, writer);
    };
    let lower =
        lower.ok_or_else(|| "big operators require a lower limit in this subset".to_string())?;
    let selector = big_op_selector(kind);
    let variation = if upper.is_some() { 0x70 } else { 0x50 };
    out.extend_from_slice(&[0x03, 0x00, selector, variation, 0x00]);
    color_default(out);
    let body_state = write_line(body, out, current_size, writer)?;
    let limit_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    if upper.is_some() {
        if body_state.size != limit_size {
            write_size(limit_size, out);
        }
        if body_state.size != limit_size || body_state.color != ColorState::Default {
            color_default(out);
        }
    }
    if upper.is_none() && body_state.color != ColorState::Default {
        color_default(out);
    }
    let lower_state = write_line(lower, out, limit_size, writer)?;
    let final_limit_state = if let Some(upper) = upper {
        if lower_state.size != limit_size {
            write_size(limit_size, out);
        }
        color_default(out);
        write_line(upper, out, limit_size, writer)?
    } else {
        if lower_state.size != limit_size {
            write_size(limit_size, out);
        }
        if lower_state.color != ColorState::Black {
            color_black(out);
        }
        write_null_line(out);
        WriteState {
            size: limit_size,
            color: ColorState::Black,
        }
    };
    out.push(0x0d);
    if final_limit_state.color != ColorState::Black {
        color_black(out);
    }
    write_big_op_glyph(kind, out)?;
    out.push(0x00);
    Ok(WriteState {
        size: limit_size,
        color: ColorState::Black,
    })
}

/// Write a big-operator template whose body slot is intentionally empty.
fn write_standalone_big_op_limits(
    kind: BigOpKind,
    lower: Option<&Expr>,
    upper: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let selector = big_op_selector(kind);
    let variation = if upper.is_some() { 0x70 } else { 0x50 };
    let limit_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    out.extend_from_slice(&[0x03, 0x00, selector, variation, 0x00]);
    color_default(out);
    write_null_line(out);
    if let Some(lower) = lower {
        write_size(limit_size, out);
        color_default(out);
        let lower_state = write_line(lower, out, limit_size, writer)?;
        if lower_state.size != limit_size {
            write_size(limit_size, out);
        }
    } else {
        write_null_line(out);
    }
    if let Some(upper) = upper {
        color_default(out);
        write_line(upper, out, limit_size, writer)?;
    } else {
        write_null_line(out);
    }
    out.push(0x0d);
    color_black(out);
    write_big_op_glyph(kind, out)?;
    out.push(0x00);
    Ok(WriteState {
        size: limit_size,
        color: ColorState::Black,
    })
}

/// Write the Sigma/Pi glyph MathType appends at the end of a big-op template.
fn write_big_op_glyph(kind: BigOpKind, out: &mut Vec<u8>) -> Result<(), String> {
    let name = big_op_glyph_name(kind);
    write_named_big_operator_glyph(name, out)
}

/// Write MathType's standalone glyph form for big operators without slots.
fn write_standalone_big_op_glyph(kind: BigOpKind, out: &mut Vec<u8>) -> Result<(), String> {
    let name = standalone_big_op_glyph_name(kind);
    write_named_big_operator_glyph(name, out)
}

/// Return MathType's template selector for large operators with limits.
fn big_op_selector(kind: BigOpKind) -> u8 {
    match kind {
        BigOpKind::Sum => 0x10,
        BigOpKind::Product => 0x11,
        BigOpKind::Coproduct => 0x12,
        BigOpKind::Union => 0x13,
        BigOpKind::Intersection => 0x14,
    }
}

/// Return the generated glyph-table key for a large operator.
fn big_op_glyph_name(kind: BigOpKind) -> &'static str {
    match kind {
        BigOpKind::Sum => "sum",
        BigOpKind::Product => "product",
        BigOpKind::Coproduct => "coproduct",
        BigOpKind::Union => "union",
        BigOpKind::Intersection => "intersection",
    }
}

/// Return the generated glyph-table key for slotless large operators.
fn standalone_big_op_glyph_name(kind: BigOpKind) -> &'static str {
    match kind {
        BigOpKind::Union => "standalone_union",
        BigOpKind::Intersection => "standalone_intersection",
        _ => big_op_glyph_name(kind),
    }
}

/// Write standalone big-symbol commands whose glyph bytes are learned by probes.
fn write_big_symbol_char(
    ch: char,
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    if let Some(name) = big_symbol_glyph_name(ch) {
        write_named_big_operator_glyph(name, out)
    } else {
        write_char(ch, out, writer)
    }
}

/// Return the generated glyph-table key for standalone big-symbol aliases.
fn big_symbol_glyph_name(ch: char) -> Option<&'static str> {
    match ch {
        '\u{22c1}' => Some("bigvee"),
        '\u{22c0}' => Some("bigwedge"),
        _ => None,
    }
}

/// Write a postfix script template; selectors match MathType sub/sup variants.
fn write_script(
    base: &Expr,
    sub: Option<&Expr>,
    sup: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if let Expr::SumOperatorSymbol(ch) = base {
        return write_sum_operator_script(*ch, sub, sup, out, current_size, writer);
    }
    let base_state = write_script_base(base, out, current_size, writer)?;
    if matches!(base, Expr::BigSymbol(_)) {
        write_size(current_size, out);
    } else if base_state.size != current_size {
        write_size(current_size, out);
    }
    color_default(out);
    let selector = match (sub.is_some(), sup.is_some()) {
        (true, false) => 0x1b,
        (false, true) => 0x1c,
        (true, true) => 0x1d,
        (false, false) => {
            return Ok(WriteState {
                size: current_size,
                color: base_state.color,
            })
        }
    };
    let script_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    out.extend_from_slice(&[0x03, 0x00, selector, 0x00, 0x00]);
    write_size(script_size, out);
    match (sub, sup) {
        (Some(sub), None) => {
            let sub_state = write_line(sub, out, script_size, writer)?;
            restore_script_separator(sub_state, script_size, out);
            write_null_line(out);
        }
        (None, Some(sup)) => {
            write_null_line(out);
            let sup_state = write_line(sup, out, script_size, writer)?;
            out.push(0x00);
            return Ok(WriteState {
                size: script_size,
                color: sup_state.color,
            });
        }
        (Some(sub), Some(sup)) => {
            let sub_state = write_line(sub, out, script_size, writer)?;
            restore_script_separator(sub_state, script_size, out);
            write_line(sup, out, script_size, writer)?;
        }
        (None, None) => {}
    }
    out.push(0x00);
    let color = if sub.is_some() && sup.is_none() {
        ColorState::Default
    } else {
        ColorState::Black
    };
    Ok(WriteState {
        size: script_size,
        color,
    })
}

/// Write MathType's tmSUMOP template used by \bigsqcup.
fn write_sum_operator_script(
    ch: char,
    sub: Option<&Expr>,
    sup: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let script_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    let variation = if sup.is_some() { 0x70 } else { 0x50 };
    out.extend_from_slice(&[0x03, 0x00, 0x16, variation, 0x00]);
    write_null_line(out);
    write_size(script_size, out);
    if let Some(sub) = sub {
        let sub_state = write_line(sub, out, script_size, writer)?;
        restore_script_separator(sub_state, script_size, out);
    } else {
        write_null_line(out);
    }
    if let Some(sup) = sup {
        write_line(sup, out, script_size, writer)?;
    } else {
        write_null_line(out);
    }
    out.push(0x0d);
    color_default(out);
    write_sum_operator_glyph_line(ch, out, writer)?;
    out.push(0x00);
    Ok(WriteState {
        size: script_size,
        color: ColorState::Black,
    })
}

/// Write the glyph slot inside a tmSUMOP template.
fn write_sum_operator_glyph_line(
    ch: char,
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    out.extend_from_slice(&[0x01, 0x00]);
    if let Some(special) = encoding::special_char(ch) {
        match special.explicit_font {
            Some(ExplicitFont::EuclidMathOne) => writer.ensure_euclid_math_one(out),
            Some(ExplicitFont::EuclidMathTwo) => writer.ensure_euclid_math_two(out),
            None => {}
        }
    }
    writer.ensure_black_color_def(out);
    color_black(out);
    write_char(ch, out, writer)?;
    out.push(0x00);
    Ok(())
}

/// Write a standalone tmSUMOP glyph when no scripts were attached.
fn write_sum_operator_glyph(
    ch: char,
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    write_char(ch, out, writer)
}

/// Write a script base before the script template slots are emitted.
fn write_script_base(
    base: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if matches!(base, Expr::BigSymbol(_)) {
        if writer.big_symbol_line_marker_pending {
            writer.big_symbol_line_marker_pending = false;
        } else {
            out.push(0x0d);
        }
    }
    write_expr(base, out, current_size, writer)
}

/// Restore size/color between script slots after nested scripts changed state.
fn restore_script_separator(state: WriteState, script_size: SizeState, out: &mut Vec<u8>) {
    if state.size != script_size {
        write_size(script_size, out);
    }
    if state.color != ColorState::Default {
        color_default(out);
    }
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
    } else if expr_starts_with_standalone_big_glyph(expr) {
        out.push(0x0d);
    }
    if starts_with_euclid_math_one || starts_with_euclid_math_two || starts_with_euclid_fraktur {
        writer.ensure_black_color_def(out);
        color_black(out);
    } else if !expr_starts_with_line_font_def(expr)
        && !expr_starts_with_raw_tex(expr)
        && !expr_starts_with_explicit_accent_template(expr)
        && !expr_starts_with_sum_operator_script_base(expr)
    {
        writer.ensure_black_color_def(out);
        color_black(out);
    }
    let final_state = write_expr(expr, out, current_size, writer)?;
    out.push(0x00);
    Ok(final_state)
}

/// Split off a leading spacing command that MathType writes before line color.
fn leading_space_rest(expr: &Expr) -> Option<(u8, &[Expr])> {
    match expr {
        Expr::Space(width) => Some((*width, &[])),
        Expr::Sequence(items) => match items.as_slice() {
            [Expr::Space(width), rest @ ..] => Some((*width, rest)),
            _ => None,
        },
        _ => None,
    }
}

/// Write a spacing CHAR without changing color, used only at the start of a LINE.
fn write_space_without_color(width: u8, out: &mut Vec<u8>) {
    out.extend_from_slice(&[0x02, 0x00, FN_SPACE, width, 0xef]);
}

/// Return true when MathType emits a font definition before the line color.
fn expr_starts_with_line_font_def(expr: &Expr) -> bool {
    match expr {
        Expr::Font {
            kind: FontKind::MathSf,
            ..
        } => true,
        Expr::Style { content, .. } => expr_starts_with_line_font_def(content),
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_line_font_def),
        _ => false,
    }
}

/// Write MathType's scalable fence template for \left...\right pairs.
fn write_delimited(
    left: char,
    right: char,
    content: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if left == '|' && right == '〉' {
        return write_ket_delimited(content, out, current_size, writer);
    }
    let selector = delimiter_selector(left, right)?;
    out.extend_from_slice(&[0x03, 0x00, selector, 0x03, 0x00]);
    color_default(out);
    let line_state = write_line(content, out, current_size, writer)?;
    if line_state.size != current_size {
        write_size(current_size, out);
    }
    if line_state.color != ColorState::Black {
        color_black(out);
    }
    write_delimiter_glyph_pair(left, right, out)?;
    out.push(0x00);
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
}

/// Write \left|...\right\rangle as MathType's Dirac ket template.
fn write_ket_delimited(
    content: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x03, 0x00, 0x1e, 0x02, 0x00]);
    // The ket variation has an omitted left slot before the visible vector.
    write_null_line(out);
    color_default(out);
    let line_state = write_line(content, out, current_size, writer)?;
    if line_state.size != current_size {
        write_size(current_size, out);
    }
    if line_state.color != ColorState::Black {
        color_black(out);
    }
    write_delimiter_glyph_pair('|', '〉', out)?;
    out.push(0x00);
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
}

/// Return the bracket template selector observed in MathType's MTEF output.
fn delimiter_selector(left: char, right: char) -> Result<u8, String> {
    match (left, right) {
        ('(', ')') => Ok(0x01),
        ('[', ']') => Ok(0x03),
        ('{', '}') => Ok(0x02),
        ('|', '|') => Ok(0x04),
        ('‖', '‖') => Ok(0x05),
        ('⌊', '⌋') => Ok(0x06),
        ('⌈', '⌉') => Ok(0x07),
        ('〈', '〉') | ('<', '>') => Ok(0x00),
        _ => Err(format!("unsupported dynamic delimiter pair: {left}{right}")),
    }
}

/// Write special paired fence glyphs whose codes differ by side.
fn write_delimiter_glyph_pair(left: char, right: char, out: &mut Vec<u8>) -> Result<(), String> {
    match (left, right) {
        ('|', '〉') => {
            write_expanding_glyph(0xec07, out);
            write_expanding_glyph(0x232a, out);
            Ok(())
        }
        ('‖', '‖') => {
            write_expanding_glyph(0xec09, out);
            write_expanding_glyph(0xec0a, out);
            Ok(())
        }
        _ => {
            write_delimiter_glyph(left, out)?;
            write_delimiter_glyph(right, out)
        }
    }
}

/// Write the explicit delimiter glyph records MathType appends to fence templates.
fn write_delimiter_glyph(ch: char, out: &mut Vec<u8>) -> Result<(), String> {
    let code = match ch {
        '⌊' => 0xf8f0,
        '⌋' => 0xf8fb,
        _ => ch as u32,
    };
    if code > u16::MAX as u32 {
        return Err(format!("delimiter is outside BMP: {ch}"));
    }
    write_expanding_glyph(code as u16, out);
    Ok(())
}
