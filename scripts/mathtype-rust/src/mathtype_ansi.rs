#[cfg(windows)]
use std::ptr;

/// Encode MathType's TeX-facing text using the active Windows ANSI code page.
///
/// MathType does not keep this future-record payload as UTF-8. On Chinese Windows, for
/// example, representable Greek letters become GBK bytes while unsupported characters fall
/// back to '?' instead of best-fit substitutions.
#[cfg(windows)]
pub(crate) fn encode_mathtype_text(text: &str) -> Result<Vec<u8>, String> {
    const CP_ACP: u32 = 0;

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

/// Keep a simple fallback for non-Windows hosts where the Win32 code-page API is absent.
#[cfg(not(windows))]
pub(crate) fn encode_mathtype_text(text: &str) -> Result<Vec<u8>, String> {
    Ok(text.as_bytes().to_vec())
}

/// Encode the MTEF TeX-source future record the same way MathType stores it.
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
