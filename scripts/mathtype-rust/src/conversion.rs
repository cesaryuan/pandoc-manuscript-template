use std::path::Path;

use crate::cfb::read_regular_stream;
use crate::mtef::{read_tex_source, write_equation_native, write_mtef, write_mtef_with_prefs};
use crate::ole::write_compound_file;
use crate::parser::{normalize_latex, Parser};

const EQUATION_NATIVE_MIN_HEADER_LEN: usize = 28;
const EQUATION_NATIVE_HEADER_LENGTH_OFFSET: usize = 0;
const EQUATION_NATIVE_MTEF_LENGTH_OFFSET: usize = 8;

/// Keep all forward-conversion products together so CLI and tests share one implementation seam.
pub(crate) struct EncodedEquation {
    pub(crate) normalized_latex: String,
    pub(crate) mtef: Vec<u8>,
    pub(crate) ole: Vec<u8>,
}

/// Convert LaTeX into normalized source, raw MTEF, and a MathType-compatible OLE object.
pub(crate) fn latex_to_equation(
    raw_latex: &str,
    prefs_file: Option<&Path>,
) -> Result<EncodedEquation, String> {
    let normalized_latex = normalize_latex(raw_latex);
    let expr = Parser::new(&normalized_latex).parse()?;
    let mtef = if let Some(path) = prefs_file {
        write_mtef_with_prefs(&normalized_latex, &expr, Some(path))?
    } else {
        write_mtef(&normalized_latex, &expr)?
    };
    let native = write_equation_native(&mtef)?;
    let ole = write_compound_file(&native)?;
    Ok(EncodedEquation {
        normalized_latex,
        mtef,
        ole,
    })
}

/// Recover LaTeX from a raw MTEF payload carrying MathType's TeX-source record.
pub(crate) fn mtef_to_latex(mtef: &[u8]) -> Result<String, String> {
    read_tex_source(mtef)?.ok_or_else(|| {
        "MTEF payload does not contain a TeX Input Language source record; structural fallback is not available"
            .to_string()
    })
}

/// Recover LaTeX from the Equation Native stream inside a MathType OLE object.
pub(crate) fn ole_to_latex(ole: &[u8]) -> Result<String, String> {
    let mtef = extract_mtef_from_ole(ole)?;
    mtef_to_latex(&mtef)
}

/// Extract and validate the raw MTEF bytes from a MathType OLE object.
pub(crate) fn extract_mtef_from_ole(ole: &[u8]) -> Result<Vec<u8>, String> {
    let equation_native = read_regular_stream(ole, "Equation Native")?;
    extract_mtef_from_equation_native(&equation_native)
}

/// Validate an Equation Native header before returning its declared MTEF payload.
fn extract_mtef_from_equation_native(equation_native: &[u8]) -> Result<Vec<u8>, String> {
    if equation_native.len() < EQUATION_NATIVE_MIN_HEADER_LEN {
        return Err("Equation Native stream is shorter than its 28-byte header".to_string());
    }
    let header_len = read_u32_at(equation_native, EQUATION_NATIVE_HEADER_LENGTH_OFFSET)? as usize;
    if header_len < EQUATION_NATIVE_MIN_HEADER_LEN {
        return Err(format!(
            "Equation Native header length is {header_len}; expected at least {EQUATION_NATIVE_MIN_HEADER_LEN}"
        ));
    }
    if header_len > equation_native.len() {
        return Err(format!(
            "Equation Native header is truncated: declared {header_len} bytes"
        ));
    }
    let length_bytes = equation_native
        .get(EQUATION_NATIVE_MTEF_LENGTH_OFFSET..EQUATION_NATIVE_MTEF_LENGTH_OFFSET + 4)
        .ok_or_else(|| "Equation Native stream is missing the MTEF length".to_string())?;
    let declared_len = u32::from_le_bytes([
        length_bytes[0],
        length_bytes[1],
        length_bytes[2],
        length_bytes[3],
    ]) as usize;
    let payload_end = header_len
        .checked_add(declared_len)
        .ok_or_else(|| "Equation Native MTEF length overflows the input".to_string())?;
    let payload = equation_native
        .get(header_len..payload_end)
        .ok_or_else(|| {
            format!("Equation Native stream is truncated: declared {declared_len} MTEF bytes")
        })?;
    Ok(payload.to_vec())
}

/// Read one little-endian `u32` from a validated fixed offset.
fn read_u32_at(data: &[u8], offset: usize) -> Result<u32, String> {
    let bytes = data
        .get(offset..offset + 4)
        .ok_or_else(|| format!("u32 offset {offset} is outside the input"))?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

#[cfg(test)]
mod tests {
    use super::{
        extract_mtef_from_equation_native, latex_to_equation, mtef_to_latex, ole_to_latex,
    };

    /// Verify both carrier forms recover the exact normalized source emitted by forward conversion.
    #[test]
    fn forward_output_round_trips_through_mtef_and_ole() {
        let equation = latex_to_equation(r"$\frac{\alpha_1}{2}$", None)
            .expect("forward conversion should succeed");

        assert_eq!(
            mtef_to_latex(&equation.mtef).expect("raw MTEF should decode"),
            equation.normalized_latex
        );
        assert_eq!(
            ole_to_latex(&equation.ole).expect("OLE should decode"),
            equation.normalized_latex
        );
    }

    /// Keep the unsupported no-source case explicit instead of returning guessed LaTeX.
    #[test]
    fn mtef_without_source_record_is_rejected() {
        let mtef = [
            0x05, 0x01, 0x00, 0x07, 0x08, b'D', b'S', b'M', b'T', b'7', 0, 1, 0x0a, 0,
        ];
        let err = mtef_to_latex(&mtef).expect_err("missing source record should fail");
        assert!(err.contains("does not contain a TeX Input Language source record"));
    }

    /// Reject inconsistent Equation Native header and payload lengths before parsing MTEF.
    #[test]
    fn malformed_equation_native_lengths_are_rejected() {
        let mut native = vec![0u8; 30];
        native[..4].copy_from_slice(&16u32.to_le_bytes());
        let err = extract_mtef_from_equation_native(&native)
            .expect_err("short declared header should fail");
        assert!(err.contains("expected at least 28"));

        native[..4].copy_from_slice(&28u32.to_le_bytes());
        native[8..12].copy_from_slice(&4u32.to_le_bytes());
        let err = extract_mtef_from_equation_native(&native)
            .expect_err("truncated declared payload should fail");
        assert!(err.contains("declared 4 MTEF bytes"));
    }
}
