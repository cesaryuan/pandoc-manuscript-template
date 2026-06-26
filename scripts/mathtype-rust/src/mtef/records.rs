use crate::typeface::FN_EXPAND;

use super::SizeState;

/// Write one generated CHAR record, preserving whether MathType used a font position.
pub(super) fn write_table_char(typeface: u8, mtcode: u16, font_pos: Option<u8>, out: &mut Vec<u8>) {
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

/// Write one fnEXPAND delimiter/accent glyph by MathType code.
pub(super) fn write_expanding_glyph(code: u16, out: &mut Vec<u8>) {
    out.push(0x02);
    out.push(0x00);
    out.push(FN_EXPAND);
    write_u16(code, out);
}

/// Write MathType's compact placeholder line for absent script slots.
pub(super) fn write_null_line(out: &mut Vec<u8>) {
    out.extend_from_slice(&[0x01, 0x01]);
}

/// Select the inherited/default color, used by MathType before template slots.
pub(super) fn color_default(out: &mut Vec<u8>) {
    out.extend_from_slice(&[0x0f, 0x00]);
}

/// Select the black color definition emitted near visible equation content.
pub(super) fn color_black(out: &mut Vec<u8>) {
    out.extend_from_slice(&[0x0f, 0x01]);
}

/// Emit a compact MathType size record when template slots need restoration.
pub(super) fn write_size(size: SizeState, out: &mut Vec<u8>) {
    out.push(match size {
        SizeState::Full => 0x0a,
        SizeState::Sub => 0x0b,
        SizeState::Sub2 => 0x0c,
    });
}

/// Write MathType's variable-length unsigned integer encoding.
pub(super) fn write_unsigned(value: usize, out: &mut Vec<u8>) -> Result<(), String> {
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
pub(super) fn write_u16(value: u16, out: &mut Vec<u8>) {
    out.extend_from_slice(&value.to_le_bytes());
}
