use crate::ast::*;
use crate::generated::char_tables::{
    EncodedChar, StyledChar, BIG_OPERATOR_GLYPHS, MATHBB_CHARS, MATHCAL_CHARS, OPERATOR_CHARS,
    SPECIAL_CHARS,
};
use crate::typeface::{
    EXPLICIT_FONT_NEG_1, EXPLICIT_FONT_NEG_2, FN_EXPAND, FN_FUNCTION, FN_MT_EXTRA, FN_NUMBER,
    FN_SPACE, FN_SYMBOL, FN_VARIABLE, FN_VECTOR,
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
}

impl MtefWriter {
    /// Emit Euclid Math One once, at the position where MathType first needs it.
    fn ensure_euclid_math_one(&mut self, out: &mut Vec<u8>) {
        if !self.euclid_math_one_defined {
            if self.euclid_math_two_defined {
                out.extend_from_slice(EUCLID_MATH_ONE_AFTER_TWO_DEFS);
            } else {
                out.extend_from_slice(EUCLID_MATH_ONE_DEFS);
            }
            self.euclid_math_one_defined = true;
        }
    }

    /// Emit Euclid Math Two once for blackboard characters such as \mathbb{I}.
    fn ensure_euclid_math_two(&mut self, out: &mut Vec<u8>) {
        if !self.euclid_math_two_defined {
            if self.euclid_math_one_defined {
                out.extend_from_slice(EUCLID_MATH_TWO_AFTER_ONE_DEFS);
            } else {
                out.extend_from_slice(EUCLID_MATH_TWO_DEFS);
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
    if let Some(body_hex) = known_environment_body_hex(source_latex) {
        out.extend_from_slice(&decode_hex(body_hex)?);
    } else {
        write_equation_body(expr, &mut out)?;
    }
    Ok(out)
}

const CASES_BODY_HEX: &str = "0a010010000000000000000f0102008368000f0003001b00000b01000f01020484b40364000f000101000a0f010200822800020083650002008229000204863d003d03000201000f00010005000100010202000001000f0103000b00000f0001000f010200883100000f0001000f010200883200000002008365000f0003001c00000b010101000f01020088320000000a0200822f00020484b403640200822c00000f0001000f010200827c0002008365000200827c000204863c003c020484b403640200822c00000f0001000f010200827c0002008365000200827c0002048612222d03000b00000f0001000f010200883100000f0001000f0102008832000000020484b403640200822c00000f0001000f010200827c0002008365000200827c000204866522b3020484b403640000000200967b00000000";

const ALIGNED_BODY_HEX: &str = "0a01000280815c0002808162000280816500028081670002808169000280816e0010000000000000000f0102008361000200836c00020083690002008367000200836e0002008365000200836400134575636c69644d617468310011074575636c6964204d617468204f6e650008060002047f12214c0f0003001b00000b01000f010200836e000200836f0002008364000200836500000f000101000a0280816e000280816e0002808126000f010204863d003d02048612222d03000b00000f0001000f010200883100000f0001000f010200834e00000003001070000f0001000f0103000303000f0001000f0102008377000f0003001c00000b010101000f010204862b002b00000a02008370000f0003001d00000b01000f010200836900000f0001000f0102008370000200837200020083650000000a0202826c000200826f00020082670003000103000f0001000f010201837000060009000f0003001d00000b01000f010200836900000f0001000f0102008370000200837200020083650000000a0204862b002b02047ff503f20002009628000200962900000204862b002b03000103000f0001000f01020088310002048612222d02008370000f0003001d00000b01000f010200836900000f0001000f010200837000020083720002008365000000000a02009628000200962900000202826c000200826f00020082670003000103000f0001000f01020088310002048612222d0201837000060009000f0003001d00000b01000f010200836900000f0001000f0102008370000200837200020083650000000a0204862b002b02047ff503f2000200962800020096290000000200965b000200965d0000000b0f0001000f0102008369000204863d003d0200883100000f0001000f010200834e00000d0204861122e5000a0f000280816e000280816e00028081260002009805ef0f010204862b002b03000b00000f0001000f010200883100000f0001000f010200834e00000003001070000f0001000f0102008368000f0003001b00000b01000f01020484b40364000f000101000001000f0102008369000204863d003d0200883100000f0001000f010200834e00000d0204861122e5000a03000103000f0001000f010201837000060009000f0003001d00000b01000f010200836900000f0001000f0102008370000200837200020083650000000a02048612222d02008370000f0003001d00000b01000f010200836900000f0001000f010200837000020083720002008365000000000a02009628000200962900000f000280816e000280816e000280815c0002808165000280816e0002808164000f0102008361000200836c00020083690002008367000200836e00020083650002008364000000";

/// Return fixed environment bodies for TeX constructs MathType handles idiosyncratically.
pub(crate) fn known_environment_body_hex(source_latex: &str) -> Option<&'static str> {
    if source_latex.contains("\\begin{cases}") {
        Some(CASES_BODY_HEX)
    } else if source_latex.contains("\\begin{aligned}") {
        Some(ALIGNED_BODY_HEX)
    } else {
        None
    }
}

/// Decode compact hex fixtures used for the two manuscript environment formulas.
fn decode_hex(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("hex fixture has an odd number of digits".to_string());
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    let mut index = 0;
    while index < hex.len() {
        let byte = u8::from_str_radix(&hex[index..index + 2], 16)
            .map_err(|err| format!("invalid hex fixture byte at {index}: {err}"))?;
        bytes.push(byte);
        index += 2;
    }
    Ok(bytes)
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
    };
    if expr_is_only_spaces(expr) {
        write_only_spaces(expr, out)?;
        out.extend_from_slice(&[0x00, 0x00]);
        return Ok(());
    } else {
        if expr_starts_with_euclid_math_one(expr) {
            writer.ensure_euclid_math_one(out);
        }
        out.extend_from_slice(&[0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        color_black(out);
    }
    write_expr(expr, out, SizeState::Full, &mut writer)?;
    out.extend_from_slice(&[0x00, 0x00]);
    Ok(())
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
                if state.color != ColorState::Black {
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
        Expr::Font { kind, content } => write_font_expr(*kind, content, out, current_size, writer)?,
        Expr::Accent { kind, content } => {
            write_accent_expr(*kind, content, out, current_size, writer)?
        }
        Expr::Fraction(numerator, denominator) => {
            write_fraction(numerator, denominator, out, current_size, writer)?
        }
        Expr::Sqrt(radicand) => write_sqrt(radicand, out, current_size, writer)?,
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
                let typeface = if writer.euclid_math_two_defined && !writer.euclid_math_one_defined
                {
                    EXPLICIT_FONT_NEG_2
                } else {
                    EXPLICIT_FONT_NEG_1
                };
                writer.ensure_euclid_math_one(out);
                write_table_char(typeface, entry.mtcode, entry.font_pos, out);
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
        FontKind::MathBb => {
            let entry = math_font_char(MATHBB_CHARS, ch, "mathbb")?;
            if entry.font_pos.is_some() && entry.typeface == EXPLICIT_FONT_NEG_1 {
                let typeface = if writer.euclid_math_one_defined {
                    EXPLICIT_FONT_NEG_2
                } else {
                    EXPLICIT_FONT_NEG_1
                };
                writer.ensure_euclid_math_two(out);
                write_table_char(typeface, entry.mtcode, entry.font_pos, out);
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
    if expr_starts_with_euclid_math_one(expr) {
        writer.ensure_euclid_math_one(out);
        color_black(out);
    } else if expr_starts_with_euclid_math_two(expr) {
        writer.ensure_euclid_math_two(out);
        color_black(out);
    } else if !expr_starts_with_line_font_def(expr) {
        color_black(out);
    }
    let final_state = write_expr(expr, out, current_size, writer)?;
    out.push(0x00);
    Ok(final_state)
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

/// Return the bracket template selector observed in MathType's MTEF output.
fn delimiter_selector(left: char, right: char) -> Result<u8, String> {
    match (left, right) {
        ('(', ')') => Ok(0x01),
        ('[', ']') => Ok(0x03),
        _ => Err(format!("unsupported dynamic delimiter pair: {left}{right}")),
    }
}

/// Write the explicit delimiter glyph records MathType appends to fence templates.
fn write_delimiter_glyph(ch: char, out: &mut Vec<u8>) -> Result<(), String> {
    let code = ch as u32;
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
