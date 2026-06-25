use crate::ast::*;
use crate::generated::char_tables::{
    EncodedChar, StyledChar, BIG_OPERATOR_GLYPHS, MATHBB_CHARS, MATHCAL_CHARS, OPERATOR_CHARS,
    SPECIAL_CHARS,
};
use crate::typeface::{
    EXPLICIT_FONT_NEG_1, EXPLICIT_FONT_NEG_2, FN_EXPAND, FN_FUNCTION, FN_MT_EXTRA, FN_NUMBER,
    FN_SPACE, FN_SYMBOL, FN_TEXT, FN_VARIABLE, FN_VECTOR,
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
    euclid_math_one_typeface: u8,
    euclid_math_two_typeface: u8,
    black_color_defined: bool,
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
            if self.euclid_math_two_defined {
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
            if self.euclid_math_one_defined {
                out.extend_from_slice(EUCLID_MATH_TWO_AFTER_ONE_DEFS);
                self.euclid_math_two_typeface = EXPLICIT_FONT_NEG_2;
            } else {
                out.extend_from_slice(EUCLID_MATH_TWO_DEFS);
                self.euclid_math_two_typeface = EXPLICIT_FONT_NEG_1;
            }
            self.euclid_math_two_defined = true;
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
        euclid_math_one_typeface: EXPLICIT_FONT_NEG_1,
        euclid_math_two_typeface: EXPLICIT_FONT_NEG_1,
        black_color_defined: false,
    };
    if expr_is_only_spaces(expr) {
        write_only_spaces(expr, out)?;
        out.extend_from_slice(&[0x00, 0x00]);
        return Ok(());
    } else {
        if expr_starts_with_euclid_math_one(expr) {
            writer.ensure_euclid_math_one(out);
        }
        if !expr_starts_with_top_matrix(expr) {
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
            kind: EnvironmentKind::Align,
            ..
        } => true,
        Expr::Sequence(items) if items.len() == 1 => expr_starts_with_top_matrix(&items[0]),
        _ => false,
    }
}

/// Return true when MathType emits Euclid Math One before the first line def.
fn expr_starts_with_euclid_math_one(expr: &Expr) -> bool {
    match expr {
        Expr::Char('ϵ') => true,
        Expr::Font {
            kind: FontKind::MathCal,
            ..
        } => true,
        Expr::Font { content, .. } | Expr::Accent { content, .. } => {
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
        Expr::Font {
            kind: FontKind::MathBb,
            ..
        } => true,
        Expr::Font { content, .. } | Expr::Accent { content, .. } => {
            expr_starts_with_euclid_math_two(content)
        }
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_euclid_math_two),
        Expr::Script { base, .. } => expr_starts_with_euclid_math_two(base),
        _ => false,
    }
}

/// Return true for algorithm-indent formulas that contain only spacing commands.
fn expr_is_only_spaces(expr: &Expr) -> bool {
    match expr {
        Expr::Space(_) => true,
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
            let mut state = WriteState {
                size: current_size,
                color: ColorState::Black,
            };
            for item in items {
                if state.size != current_size {
                    write_size(current_size, out);
                    state.size = current_size;
                }
                if state.color != ColorState::Black && !expr_sets_own_color(item) {
                    color_black(out);
                    state.color = ColorState::Black;
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
        Expr::Font { kind, content } => write_font_expr(*kind, content, out, current_size, writer)?,
        Expr::Accent { kind, content } => {
            write_accent_expr(*kind, content, out, current_size, writer)?
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
            write_integral(*kind, out);
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
        Expr::Binomial(upper, lower) => {
            write_binomial(upper, lower, out, current_size, writer)?;
            WriteState {
                size: current_size,
                color: ColorState::Black,
            }
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
    matches!(expr, Expr::Color { .. })
}

/// Write one MTEF CHAR record using MathType's simple font/style choices.
fn write_char(ch: char, out: &mut Vec<u8>, writer: &mut MtefWriter) -> Result<(), String> {
    if ch == 'ϵ' {
        writer.ensure_euclid_math_one(out);
    }
    if let Some(special) = special_char(ch) {
        write_table_char(special.typeface, special.mtcode, special.font_pos, out);
        return Ok(());
    }

    let code = ch as u32;
    if code > u16::MAX as u32 {
        return Err(format!(
            "character is outside BMP and not yet supported: {ch}"
        ));
    }

    if let Some(operator) = encoded_char(OPERATOR_CHARS, ch) {
        write_table_char(operator.typeface, operator.mtcode, operator.font_pos, out);
    } else if is_function_char(ch) {
        out.push(0x02);
        out.push(0x00);
        out.push(FN_FUNCTION);
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

/// Return MathType's generated style/font-position tuple for TeX command symbols.
fn special_char(ch: char) -> Option<StyledChar> {
    SPECIAL_CHARS.iter().copied().find(|entry| entry.ch == ch)
}

/// Return a generated character entry from a table.
fn encoded_char(table: &[EncodedChar], ch: char) -> Option<EncodedChar> {
    table.iter().copied().find(|entry| entry.ch == ch)
}

/// Write one generated CHAR record, preserving whether MathType used a font position.
fn write_table_char(typeface: u8, mtcode: u16, font_pos: Option<u8>, out: &mut Vec<u8>) {
    out.push(0x02);
    if let Some(font_pos) = font_pos {
        out.push(0x04);
        out.push(typeface);
        write_u16(mtcode, out);
        out.push(font_pos);
    } else {
        out.push(0x00);
        out.push(typeface);
        write_u16(mtcode, out);
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
    for ch in text.chars() {
        let code = ch as u32;
        if code > u16::MAX as u32 {
            return Err(format!("text character is outside BMP: {ch}"));
        }
        out.push(0x02);
        out.push(0x00);
        out.push(FN_TEXT);
        write_u16(code as u16, out);
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
    let lower =
        lower.ok_or_else(|| "integrals require a lower limit in this subset".to_string())?;
    let variation = match kind {
        IntegralKind::Single => 0x11,
        IntegralKind::Double => 0x12,
        IntegralKind::Contour => 0x15,
    };
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
    if body_state.color != ColorState::Default {
        color_default(out);
    }
    let lower_state = write_line(lower, out, limit_size, writer)?;
    if lower_state.size != limit_size {
        write_size(limit_size, out);
    }
    if lower_state.color != ColorState::Black {
        color_black(out);
    }
    if let Some(upper) = upper {
        write_line(upper, out, limit_size, writer)?;
    } else {
        write_null_line(out);
    }
    out.push(0x0d);
    if kind == IntegralKind::Contour {
        write_named_big_operator_glyph("contour_loop", out)?;
    }
    write_integral_glyph(out)?;
    out.push(0x00);
    Ok(WriteState {
        size: limit_size,
        color: ColorState::Black,
    })
}

/// Write the integral glyph MathType appends at the end of integral templates.
fn write_integral_glyph(out: &mut Vec<u8>) -> Result<(), String> {
    write_named_big_operator_glyph("integral", out)
}

/// Write a generated glyph used by MathType's big-operator templates.
fn write_named_big_operator_glyph(name: &str, out: &mut Vec<u8>) -> Result<(), String> {
    let glyph = BIG_OPERATOR_GLYPHS
        .iter()
        .find(|glyph| glyph.name == name)
        .ok_or_else(|| format!("missing generated big-operator glyph: {name}"))?;
    out.push(0x02);
    out.push(0x04);
    out.push(if name == "contour_loop" {
        FN_MT_EXTRA
    } else {
        FN_SYMBOL
    });
    write_u16(glyph.mtcode, out);
    out.push(glyph.font_pos);
    Ok(())
}

/// Write a supported integral glyph as a CHAR record.
fn write_integral(kind: IntegralKind, out: &mut Vec<u8>) {
    let ch = match kind {
        IntegralKind::Single => '∫',
        IntegralKind::Double => '∬',
        IntegralKind::Contour => '∮',
    };
    out.push(0x02);
    out.push(0x00);
    out.push(FN_SYMBOL);
    write_u16(ch as u16, out);
}

/// Write a binomial as a parenthesized stacked fraction-like form.
fn write_binomial(
    upper: &Expr,
    lower: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    write_char('(', out, writer)?;
    write_fraction(upper, lower, out, current_size, writer)?;
    write_char(')', out, writer)?;
    Ok(())
}

/// Write a matrix environment as a real MTEF MATRIX wrapped in its fence template.
fn write_matrix(
    kind: MatrixKind,
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    let (left, right, selector) = match kind {
        MatrixKind::Parenthesized => ('(', ')', 0x01),
        MatrixKind::Bracketed => ('[', ']', 0x03),
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
        EnvironmentKind::Align => write_align_matrix_record(rows, out, current_size, writer),
        EnvironmentKind::Aligned => write_matrix_record(rows, out, current_size, writer),
        EnvironmentKind::Cases => write_left_fenced_matrix(rows, out, current_size, writer),
    }
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
    write_matrix_slot_line(rows, out, current_size, writer)?;
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
    write_matrix_slot_line(rows, out, current_size, writer)?;
    write_delimiter_glyph('{', out)?;
    out.push(0x00);
    Ok(())
}

/// Write a template slot line that contains only one MATRIX object.
fn write_matrix_slot_line(
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    out.extend_from_slice(&[0x01, 0x00]);
    write_matrix_record(rows, out, current_size, writer)?;
    out.push(0x00);
    Ok(())
}

/// Write MathType's MATRIX record and one LINE object for each cell.
fn write_matrix_record(
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    let row_count = u8::try_from(rows.len()).map_err(|_| "matrix has too many rows".to_string())?;
    let col_count = rows.iter().map(Vec::len).max().unwrap_or(0);
    let col_count =
        u8::try_from(col_count).map_err(|_| "matrix has too many columns".to_string())?;

    out.extend_from_slice(&[0x05, 0x00, 0x01, 0x00, 0x01, row_count, col_count]);
    out.extend(std::iter::repeat(0x00).take(partition_byte_count(row_count)));
    out.extend(std::iter::repeat(0x00).take(partition_byte_count(col_count)));
    let mut cell_ordinal = 0usize;
    let mut previous_cell_was_empty = false;
    for row in rows {
        for col_index in 0..col_count as usize {
            if cell_ordinal > 0 && !previous_cell_was_empty {
                color_default(out);
            }
            if let Some(cell) = row.get(col_index) {
                previous_cell_was_empty = expr_is_empty_sequence(cell);
                write_matrix_cell_line(cell, out, current_size, writer)?;
            } else {
                previous_cell_was_empty = true;
                write_empty_matrix_cell_line(out);
            }
            cell_ordinal += 1;
        }
    }
    out.push(0x00);
    Ok(())
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
) -> Result<(), String> {
    if expr_is_empty_sequence(expr) {
        write_empty_matrix_cell_line(out);
        return Ok(());
    }
    write_line(expr, out, current_size, writer)?;
    Ok(())
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
            let entry = math_font_char(MATHCAL_CHARS, ch, "mathcal")?;
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
            let entry = math_font_char(MATHCAL_CHARS, ch, "mathscr")?;
            writer.ensure_euclid_math_one(out);
            write_table_char(
                writer.euclid_math_one_typeface,
                entry.mtcode,
                entry.font_pos,
                out,
            );
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
        FontKind::MathBb => {
            let entry = math_font_char(MATHBB_CHARS, ch, "mathbb")?;
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

/// Return one generated math-font mapping or a font-specific error.
fn math_font_char(table: &[EncodedChar], ch: char, font_name: &str) -> Result<EncodedChar, String> {
    encoded_char(table, ch).ok_or_else(|| format!("unsupported {font_name} character: {ch}"))
}

/// Write simple MathType embellishments such as \bar{I} and \hat{P}.
fn write_accent_expr(
    kind: AccentKind,
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if kind == AccentKind::Vec {
        return write_vector_template(expr, out, current_size, writer);
    }
    if let Some((font_kind, ch)) = single_font_char(expr) {
        match (kind, font_kind) {
            (AccentKind::Bar | AccentKind::Hat, None) => {
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
    if let Some(ch) = widehat_bar_char(expr) {
        write_embellished_char(ch, &[AccentKind::Bar, AccentKind::Hat], out)?;
        return Ok(WriteState {
            size: current_size,
            color: ColorState::Black,
        });
    }
    write_expr(expr, out, current_size, writer)
}

/// Write MathType's vector-arrow template for \vec{...}.
fn write_vector_template(
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x03, 0x00, 0x1f, 0x02, 0x00]);
    color_default(out);
    let line_state = write_line(expr, out, current_size, writer)?;
    if line_state.size != current_size {
        write_size(current_size, out);
    }
    if line_state.color != ColorState::Black {
        color_black(out);
    }
    write_delimiter_glyph('\u{20d7}', out)?;
    out.push(0x00);
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
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
            euclid_math_one_typeface: EXPLICIT_FONT_NEG_1,
            euclid_math_two_typeface: EXPLICIT_FONT_NEG_2,
            black_color_defined: true,
        },
    )?;
    out.push(0x00);
    out.extend_from_slice(&[0x02, 0x00, FN_EXPAND, 0x02, 0x03, 0x00]);
    Ok(())
}

/// Write a CHAR record with one or more embellishments attached.
fn write_embellished_char(ch: char, kinds: &[AccentKind], out: &mut Vec<u8>) -> Result<(), String> {
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
    for kind in kinds {
        out.extend_from_slice(&[
            0x06,
            0x00,
            match kind {
                AccentKind::Hat | AccentKind::WideHat => 0x09,
                AccentKind::Bar => 0x11,
                AccentKind::Vec => 0x09,
            },
        ]);
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
    let body = body.ok_or_else(|| "internal error: missing big-operator operand".to_string())?;
    let lower =
        lower.ok_or_else(|| "big operators require a lower limit in this subset".to_string())?;
    let selector = match kind {
        BigOpKind::Sum => 0x10,
        BigOpKind::Product => 0x11,
    };
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

/// Write the Sigma/Pi glyph MathType appends at the end of a big-op template.
fn write_big_op_glyph(kind: BigOpKind, out: &mut Vec<u8>) -> Result<(), String> {
    let name = match kind {
        BigOpKind::Sum => "sum",
        BigOpKind::Product => "product",
    };
    let glyph = BIG_OPERATOR_GLYPHS
        .iter()
        .find(|glyph| glyph.name == name)
        .ok_or_else(|| format!("missing generated big-operator glyph: {name}"))?;
    out.push(0x02);
    out.push(0x04);
    out.push(FN_SYMBOL);
    write_u16(glyph.mtcode, out);
    out.push(glyph.font_pos);
    Ok(())
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
    let base_state = write_expr(base, out, current_size, writer)?;
    if base_state.size != current_size {
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
    if expr_starts_with_euclid_math_one(expr) {
        writer.ensure_euclid_math_one(out);
        writer.ensure_black_color_def(out);
        color_black(out);
    } else if expr_starts_with_euclid_math_two(expr) {
        writer.ensure_euclid_math_two(out);
        writer.ensure_black_color_def(out);
        color_black(out);
    } else if !expr_starts_with_line_font_def(expr) {
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
    write_delimiter_glyph(left, out)?;
    write_delimiter_glyph(right, out)?;
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
    out.extend_from_slice(&[0x03, 0x00, 0x1e, 0x02]);
    color_default(out);
    let line_state = write_line(content, out, current_size, writer)?;
    if line_state.size != current_size {
        write_size(current_size, out);
    }
    if line_state.color != ColorState::Black {
        color_black(out);
    }
    write_delimiter_glyph('|', out)?;
    write_delimiter_glyph('〉', out)?;
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
        ('⌊', '⌋') => Ok(0x06),
        ('⌈', '⌉') => Ok(0x07),
        ('〈', '〉') => Ok(0x00),
        _ => Err(format!("unsupported dynamic delimiter pair: {left}{right}")),
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
    out.push(0x02);
    out.push(0x00);
    out.push(FN_EXPAND);
    write_u16(code as u16, out);
    Ok(())
}

/// Write MathType's compact placeholder line for absent script slots.
fn write_null_line(out: &mut Vec<u8>) {
    out.extend_from_slice(&[0x01, 0x01]);
}

/// Select the inherited/default color, used by MathType before template slots.
fn color_default(out: &mut Vec<u8>) {
    out.extend_from_slice(&[0x0f, 0x00]);
}

/// Select the black color definition emitted near visible equation content.
fn color_black(out: &mut Vec<u8>) {
    out.extend_from_slice(&[0x0f, 0x01]);
}

/// Emit a compact MathType size record when template slots need restoration.
fn write_size(size: SizeState, out: &mut Vec<u8>) {
    out.push(match size {
        SizeState::Full => 0x0a,
        SizeState::Sub => 0x0b,
        SizeState::Sub2 => 0x0c,
    });
}

/// Write MathType's variable-length unsigned integer encoding.
fn write_unsigned(value: usize, out: &mut Vec<u8>) -> Result<(), String> {
    if value < 255 {
        out.push(value as u8);
    } else if value <= u16::MAX as usize {
        out.push(255);
        write_u16(value as u16, out);
    } else {
        return Err(format!(
            "value is too large for MTEF unsigned integer: {value}"
        ));
    }
    Ok(())
}

/// Write a little-endian 16-bit value.
fn write_u16(value: u16, out: &mut Vec<u8>) {
    out.extend_from_slice(&value.to_le_bytes());
}
