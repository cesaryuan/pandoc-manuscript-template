use crate::mathtype_ansi::{decode_mathtype_source, encode_mathtype_source};

const MTEF_VERSION: u8 = 5;
const WINDOWS_PLATFORM: u8 = 1;
const MATHTYPE_PRODUCT: u8 = 0;
const PRODUCT_VERSION: u8 = 7;
const PRODUCT_SUBVERSION: u8 = 8;
const APPLICATION_KEY: &[u8] = b"DSMT7\0";
const TEX_SOURCE_RECORD_TYPE: u8 = 0x66;
const TEX_TRANSLATOR_NAME: &[u8] = b"TeX Input Language";

/// Write the stable MTEF v5 header used by both forward conversion and reverse parsing.
pub(super) fn write_header(inline: bool, out: &mut Vec<u8>) {
    out.extend_from_slice(&[
        MTEF_VERSION,
        WINDOWS_PLATFORM,
        MATHTYPE_PRODUCT,
        PRODUCT_VERSION,
        PRODUCT_SUBVERSION,
    ]);
    out.extend_from_slice(APPLICATION_KEY);
    out.push(u8::from(inline));
}

/// Write MathType's TeX-source future record so a generated equation remains reversible.
pub(super) fn write_tex_source_record(source_latex: &str, out: &mut Vec<u8>) -> Result<(), String> {
    let mut payload = TEX_TRANSLATOR_NAME.to_vec();
    payload.push(0);
    payload.extend_from_slice(&encode_mathtype_source(source_latex)?);
    payload.push(0);

    out.push(TEX_SOURCE_RECORD_TYPE);
    write_unsigned(payload.len(), out)?;
    out.extend_from_slice(&payload);
    Ok(())
}

/// Decode the TeX source stored in an MTEF future record, when one is present.
pub(crate) fn read_tex_source(mtef: &[u8]) -> Result<Option<String>, String> {
    let mut cursor = parse_header(mtef)?;
    while let Some(&record_type) = mtef.get(cursor) {
        if record_type < 100 {
            return Ok(None);
        }
        cursor += 1;
        let payload_len = read_unsigned(mtef, &mut cursor)?;
        let payload_end = cursor
            .checked_add(payload_len)
            .ok_or_else(|| "MTEF future-record length overflows the input".to_string())?;
        let payload = mtef.get(cursor..payload_end).ok_or_else(|| {
            format!("MTEF future record 0x{record_type:02x} is truncated: need {payload_len} bytes")
        })?;
        cursor = payload_end;

        if record_type == TEX_SOURCE_RECORD_TYPE {
            if let Some(source) = decode_tex_source_payload(payload)? {
                return Ok(Some(source));
            }
        }
    }
    Ok(None)
}

/// Parse and validate the fixed-width and null-terminated parts of an MTEF v5 header.
fn parse_header(mtef: &[u8]) -> Result<usize, String> {
    if mtef.len() < 7 {
        return Err("MTEF payload is too short to contain a version 5 header".to_string());
    }
    if mtef[0] != MTEF_VERSION {
        return Err(format!(
            "unsupported MTEF version {}; expected version {MTEF_VERSION}",
            mtef[0]
        ));
    }

    let app_key_start = 5usize;
    let app_key_end = mtef[app_key_start..]
        .iter()
        .position(|byte| *byte == 0)
        .map(|offset| app_key_start + offset)
        .ok_or_else(|| "MTEF application key is not null-terminated".to_string())?;
    let equation_options = app_key_end + 1;
    if equation_options >= mtef.len() {
        return Err("MTEF header is missing the equation-options byte".to_string());
    }
    Ok(equation_options + 1)
}

/// Decode one TeX-source record payload after verifying its translator identifier.
fn decode_tex_source_payload(payload: &[u8]) -> Result<Option<String>, String> {
    let translator_end = payload
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(|| "MTEF TeX-source record has no translator terminator".to_string())?;
    if &payload[..translator_end] != TEX_TRANSLATOR_NAME {
        return Ok(None);
    }

    let source_start = translator_end + 1;
    let source_end = payload[source_start..]
        .iter()
        .position(|byte| *byte == 0)
        .map(|offset| source_start + offset)
        .ok_or_else(|| "MTEF TeX-source record has no source terminator".to_string())?;
    decode_mathtype_source(&payload[source_start..source_end]).map(Some)
}

/// Write MathType's variable-length unsigned integer encoding.
fn write_unsigned(value: usize, out: &mut Vec<u8>) -> Result<(), String> {
    if value < 255 {
        out.push(value as u8);
    } else if value <= u16::MAX as usize {
        out.push(255);
        out.extend_from_slice(&(value as u16).to_le_bytes());
    } else {
        return Err(format!(
            "value is too large for an MTEF unsigned integer: {value}"
        ));
    }
    Ok(())
}

/// Read MathType's variable-length unsigned integer encoding.
fn read_unsigned(data: &[u8], cursor: &mut usize) -> Result<usize, String> {
    let first = *data
        .get(*cursor)
        .ok_or_else(|| "MTEF unsigned integer is truncated".to_string())?;
    *cursor += 1;
    if first < 255 {
        return Ok(first as usize);
    }

    let bytes = data
        .get(*cursor..*cursor + 2)
        .ok_or_else(|| "MTEF extended unsigned integer is truncated".to_string())?;
    *cursor += 2;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]) as usize)
}

#[cfg(test)]
mod tests {
    use super::{read_tex_source, write_header, write_tex_source_record};

    /// Keep source-record encoding and decoding exactly reversible for supported text.
    #[test]
    fn tex_source_record_round_trips() {
        let source = r"\frac{\alpha_1}{2}";
        let mut mtef = Vec::new();
        write_header(true, &mut mtef);
        write_tex_source_record(source, &mut mtef).expect("source record should encode");
        mtef.push(0);

        assert_eq!(
            read_tex_source(&mtef).expect("source record should decode"),
            Some(source.to_string())
        );
    }

    /// A valid MTEF equation without a TeX future record must remain distinguishable.
    #[test]
    fn missing_tex_source_record_returns_none() {
        let mut mtef = Vec::new();
        write_header(true, &mut mtef);
        mtef.extend_from_slice(&[0x0a, 0x00]);

        assert_eq!(
            read_tex_source(&mtef).expect("header should remain readable"),
            None
        );
    }
}
