#[cfg(windows)]
use std::ptr;

use encoding_rs::{EncoderResult, Encoding, GBK, UTF_8, WINDOWS_1252};

const SOURCE_ENCODING_ENV: &str = "MATHTYPE_RUST_SOURCE_ENCODING";

/// Encode MathType's TeX-facing text using the active Windows ANSI code page.
///
/// MathType does not keep this future-record payload as UTF-8. On Chinese Windows, for
/// example, representable Greek letters become GBK bytes while other characters may use
/// Windows best-fit mappings or fall back to question marks.
#[cfg(windows)]
pub(crate) fn encode_mathtype_text(text: &str) -> Result<Vec<u8>, String> {
    const CP_ACP: u32 = 0;

    if let Some(encoding) = configured_source_encoding()? {
        return encode_with_replacement(encoding, text);
    }

    let wide = text.encode_utf16().collect::<Vec<_>>();
    if wide.is_empty() {
        return Ok(Vec::new());
    }
    let default_char = b'?';
    let wide_len = i32::try_from(wide.len())
        .map_err(|_| "TeX source is too large for Windows code-page conversion".to_string())?;
    if let Some(bytes) = encode_windows_code_page(&wide, wide_len, CP_ACP, 0, &default_char)? {
        return Ok(bytes);
    }
    if let Some(bytes) =
        encode_windows_code_page(&wide, wide_len, CP_ACP, 0x0000_0400, &default_char)?
    {
        return Ok(bytes);
    }

    let mut bytes = Vec::new();
    for ch in text.chars() {
        let chunk = match encode_mathtype_char(ch, CP_ACP, 0, &default_char) {
            Ok(Some(bytes)) => bytes,
            Ok(None) | Err(_) => match encode_mathtype_char(ch, CP_ACP, 0x0000_0400, &default_char)
            {
                Ok(Some(bytes)) => bytes,
                Ok(None) | Err(_) => vec![b'?'],
            },
        };
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

/// Use a portable configured encoding on hosts where the Win32 code-page API is absent.
#[cfg(not(windows))]
pub(crate) fn encode_mathtype_text(text: &str) -> Result<Vec<u8>, String> {
    let encoding = configured_source_encoding()?.unwrap_or(UTF_8);
    encode_with_replacement(encoding, text)
}

/// Decode MathType's TeX-source bytes using the active Windows ANSI code page.
#[cfg(windows)]
#[allow(dead_code)]
pub(crate) fn decode_mathtype_source(bytes: &[u8]) -> Result<String, String> {
    const CP_ACP: u32 = 0;

    if let Some(encoding) = configured_source_encoding()? {
        return decode_without_replacement(encoding, bytes);
    }

    if bytes.is_empty() {
        return Ok(String::new());
    }
    let byte_len = i32::try_from(bytes.len())
        .map_err(|_| "MTEF TeX source is too large for Windows code-page conversion".to_string())?;
    let wide = decode_windows_code_page(bytes, byte_len, CP_ACP)?;
    String::from_utf16(&wide).map_err(|err| format!("invalid decoded MTEF TeX source: {err}"))
}

/// Decode the UTF-8-compatible source representation used on non-Windows hosts.
#[cfg(not(windows))]
#[allow(dead_code)]
pub(crate) fn decode_mathtype_source(bytes: &[u8]) -> Result<String, String> {
    let encoding = configured_source_encoding()?.unwrap_or(UTF_8);
    decode_without_replacement(encoding, bytes)
}

/// Resolve the optional deterministic source encoding used by CI and cross-platform tools.
fn configured_source_encoding() -> Result<Option<&'static Encoding>, String> {
    let Some(value) = std::env::var_os(SOURCE_ENCODING_ENV) else {
        return Ok(None);
    };
    let value = value.to_string_lossy();
    let normalized = value.trim().to_ascii_lowercase();
    let encoding = match normalized.as_str() {
        "utf-8" | "utf8" => UTF_8,
        "gbk" | "gb18030" | "windows-936" | "cp936" => GBK,
        "windows-1252" | "cp1252" => WINDOWS_1252,
        _ => {
            return Err(format!(
                "unsupported {SOURCE_ENCODING_ENV} value {value:?}; expected utf-8, gbk, or windows-1252"
            ))
        }
    };
    Ok(Some(encoding))
}

/// Encode text with MathType's question-mark fallback for unrepresentable characters.
fn encode_with_replacement(encoding: &'static Encoding, text: &str) -> Result<Vec<u8>, String> {
    if encoding == UTF_8 {
        return Ok(text.as_bytes().to_vec());
    }

    let mut encoder = encoding.new_encoder();
    let mut remaining = text;
    let mut output = Vec::new();
    loop {
        let input = remaining;
        let reserve = encoder
            .max_buffer_length_from_utf8_without_replacement(input.len())
            .ok_or_else(|| "MTEF TeX source encoded size overflows memory limits".to_string())?;
        output.reserve(reserve);
        let (result, read) =
            encoder.encode_from_utf8_to_vec_without_replacement(input, &mut output, true);
        remaining = &input[read..];
        match result {
            EncoderResult::InputEmpty => return Ok(output),
            EncoderResult::Unmappable(ch) => {
                // Win32 ACP conversion replaces each unsupported UTF-16 code unit,
                // so a non-BMP scalar becomes two question marks in MathType records.
                output.push(b'?');
                if ch.len_utf16() == 2 {
                    output.push(b'?');
                }
                if remaining.starts_with(ch) {
                    remaining = &remaining[ch.len_utf8()..];
                } else if !input[..read].ends_with(ch) {
                    return Err(
                        "source encoder reported an invalid unmappable-character offset"
                            .to_string(),
                    );
                }
            }
            EncoderResult::OutputFull => {
                return Err("source encoder exhausted a pre-sized output buffer".to_string())
            }
        }
    }
}

/// Decode one configured source encoding without silently replacing malformed byte sequences.
fn decode_without_replacement(encoding: &'static Encoding, bytes: &[u8]) -> Result<String, String> {
    encoding
        .decode_without_bom_handling_and_without_replacement(bytes)
        .map(|text| text.into_owned())
        .ok_or_else(|| {
            format!(
                "invalid MTEF TeX source for {}; set {SOURCE_ENCODING_ENV} to the source Windows code page",
                encoding.name()
            )
        })
}

/// Encode the MTEF TeX-source future record the same way MathType stores it.
/// Some helper binaries only need text-mode encoding, so keep this shared source encoder
/// available without forcing every crate target to reference it directly.
#[allow(dead_code)]
pub(crate) fn encode_mathtype_source(text: &str) -> Result<Vec<u8>, String> {
    encode_mathtype_text(text)
}

#[cfg(windows)]
fn encode_windows_code_page(
    wide: &[u16],
    wide_len: i32,
    code_page: u32,
    flags: u32,
    default_char: &u8,
) -> Result<Option<Vec<u8>>, String> {
    unsafe extern "system" {
        fn WideCharToMultiByte(
            code_page: u32,
            flags: u32,
            wide_str: *const u16,
            wide_len: i32,
            multi_str: *mut u8,
            multi_len: i32,
            default_char: *const u8,
            used_default_char: *mut i32,
        ) -> i32;
    }

    let size = unsafe {
        // Ask Windows for the exact ANSI byte count using the caller's conversion policy.
        WideCharToMultiByte(
            code_page,
            flags,
            wide.as_ptr(),
            wide_len,
            ptr::null_mut(),
            0,
            default_char,
            ptr::null_mut(),
        )
    };
    if size <= 0 {
        return Ok(None);
    }

    let mut bytes = vec![0u8; size as usize];
    let written = unsafe {
        // Convert with the same active ANSI code page and caller-selected fallback behavior.
        WideCharToMultiByte(
            code_page,
            flags,
            wide.as_ptr(),
            wide_len,
            bytes.as_mut_ptr(),
            size,
            default_char,
            ptr::null_mut(),
        )
    };
    if written != size {
        return Ok(None);
    }
    Ok(Some(bytes))
}

#[cfg(windows)]
#[allow(dead_code)]
fn decode_windows_code_page(
    bytes: &[u8],
    byte_len: i32,
    code_page: u32,
) -> Result<Vec<u16>, String> {
    unsafe extern "system" {
        fn MultiByteToWideChar(
            code_page: u32,
            flags: u32,
            multi_str: *const u8,
            multi_len: i32,
            wide_str: *mut u16,
            wide_len: i32,
        ) -> i32;
    }

    let size = unsafe {
        // Query the exact UTF-16 length using the same active code page as the writer.
        MultiByteToWideChar(code_page, 0, bytes.as_ptr(), byte_len, ptr::null_mut(), 0)
    };
    if size <= 0 {
        return Err("failed to measure MTEF TeX source in the active code page".to_string());
    }

    let mut wide = vec![0u16; size as usize];
    let written = unsafe {
        // Decode with the active code page so source records mirror MathType on this host.
        MultiByteToWideChar(
            code_page,
            0,
            bytes.as_ptr(),
            byte_len,
            wide.as_mut_ptr(),
            size,
        )
    };
    if written != size {
        return Err("failed to decode the complete MTEF TeX source".to_string());
    }
    Ok(wide)
}

#[cfg(windows)]
fn encode_mathtype_char(
    ch: char,
    code_page: u32,
    flags: u32,
    default_char: &u8,
) -> Result<Option<Vec<u8>>, String> {
    let mut buf = [0u16; 2];
    let wide = ch.encode_utf16(&mut buf);
    let wide_len = i32::try_from(wide.len())
        .map_err(|_| "TeX character is too large for Windows code-page conversion".to_string())?;
    encode_windows_code_page(wide, wide_len, code_page, flags, default_char)
}

#[cfg(test)]
mod tests {
    use super::{decode_without_replacement, encode_with_replacement};
    use encoding_rs::GBK;

    /// Keep the deterministic GBK path compatible with the Chinese-Windows sample corpus.
    #[test]
    fn gbk_source_encoding_round_trips_representable_text() {
        let source = r"\alpha+α";
        let bytes = encode_with_replacement(GBK, source).expect("GBK encoding should succeed");
        assert_eq!(
            decode_without_replacement(GBK, &bytes).expect("GBK decoding should succeed"),
            source
        );
    }

    /// Preserve MathType's literal question-mark fallback for characters outside the code page.
    #[test]
    fn gbk_source_encoding_replaces_unrepresentable_text() {
        let bytes = encode_with_replacement(GBK, "𝔄").expect("GBK encoding should succeed");
        assert_eq!(bytes, b"??");
    }
}
