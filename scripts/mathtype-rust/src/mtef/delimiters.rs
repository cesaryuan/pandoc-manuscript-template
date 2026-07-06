use super::*;

/// Write MathType's scalable fence template for \left...\right pairs.
pub(super) fn write_delimited(
    left: char,
    right: char,
    content: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if left == '|' && right == '\u{3009}' {
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

/// Write MathType's one-sided scalable fence templates such as `\left.A\right)`.
pub(super) fn write_one_sided_delimited(
    side: OneSidedDelimiterSide,
    delimiter: char,
    content: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let selector = one_sided_delimiter_selector(delimiter)?;
    let variation = match side {
        OneSidedDelimiterSide::LeftVisible => 0x01,
        OneSidedDelimiterSide::RightVisible => 0x02,
    };
    out.extend_from_slice(&[0x03, 0x00, selector, variation, 0x00]);
    color_default(out);
    let line_state = write_line(content, out, current_size, writer)?;
    if line_state.size != current_size {
        write_size(current_size, out);
    }
    if line_state.color != ColorState::Black {
        color_black(out);
    }
    write_one_sided_delimiter_glyph(side, delimiter, out)?;
    out.push(0x00);
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
}

/// Write \left|...\right\rangle as MathType's Dirac ket template.
pub(super) fn write_ket_delimited(
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
    write_delimiter_glyph_pair('|', '\u{3009}', out)?;
    out.push(0x00);
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
}

/// Return the bracket template selector observed in MathType's MTEF output.
pub(super) fn delimiter_selector(left: char, right: char) -> Result<u8, String> {
    match (left, right) {
        ('(', ')') => Ok(0x01),
        ('[', ']') => Ok(0x03),
        ('{', '}') => Ok(0x02),
        ('|', '|') => Ok(0x04),
        ('\u{2016}', '\u{2016}') => Ok(0x05),
        ('\u{230a}', '\u{230b}') => Ok(0x06),
        ('\u{2308}', '\u{2309}') => Ok(0x07),
        ('\u{3008}', '\u{3009}') | ('<', '>') => Ok(0x00),
        _ => Err(format!("unsupported dynamic delimiter pair: {left}{right}")),
    }
}

/// Write the visible fence glyph for a one-sided delimiter template.
fn write_one_sided_delimiter_glyph(
    side: OneSidedDelimiterSide,
    delimiter: char,
    out: &mut Vec<u8>,
) -> Result<(), String> {
    match (side, delimiter) {
        (OneSidedDelimiterSide::LeftVisible, '|') => {
            write_expanding_glyph(0xec07, out);
            Ok(())
        }
        (OneSidedDelimiterSide::RightVisible, '|') => {
            write_expanding_glyph(0xec08, out);
            Ok(())
        }
        (OneSidedDelimiterSide::LeftVisible, '\u{2016}') => {
            write_expanding_glyph(0xec09, out);
            Ok(())
        }
        (OneSidedDelimiterSide::RightVisible, '\u{2016}') => {
            write_expanding_glyph(0xec0a, out);
            Ok(())
        }
        _ => write_delimiter_glyph(delimiter, out),
    }
}

/// Return the fence selector for one-sided delimiters that still use MathType's native template.
fn one_sided_delimiter_selector(delimiter: char) -> Result<u8, String> {
    match delimiter {
        '(' | ')' => Ok(0x01),
        '[' | ']' => Ok(0x03),
        '{' | '}' => Ok(0x02),
        '|' => Ok(0x04),
        '\u{2016}' => Ok(0x05),
        '\u{230a}' | '\u{230b}' => Ok(0x06),
        '\u{2308}' | '\u{2309}' => Ok(0x07),
        '\u{3008}' | '\u{3009}' | '<' | '>' => Ok(0x00),
        _ => Err(format!("unsupported one-sided delimiter: {delimiter}")),
    }
}

/// Write special paired fence glyphs whose codes differ by side.
pub(super) fn write_delimiter_glyph_pair(
    left: char,
    right: char,
    out: &mut Vec<u8>,
) -> Result<(), String> {
    match (left, right) {
        ('|', '\u{3009}') => {
            write_expanding_glyph(0xec07, out);
            write_expanding_glyph(0x232a, out);
            Ok(())
        }
        ('|', '|') => {
            write_expanding_glyph(0xec07, out);
            write_expanding_glyph(0xec08, out);
            Ok(())
        }
        ('\u{2016}', '\u{2016}') => {
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
pub(super) fn write_delimiter_glyph(ch: char, out: &mut Vec<u8>) -> Result<(), String> {
    let code = match ch {
        '\u{230a}' => 0xf8f0,
        '\u{230b}' => 0xf8fb,
        '\u{2308}' => 0xf8f0,
        '\u{2309}' => 0xf8fb,
        _ => ch as u32,
    };
    if code > u16::MAX as u32 {
        return Err(format!("delimiter is outside BMP: {ch}"));
    }
    write_expanding_glyph(code as u16, out);
    Ok(())
}
