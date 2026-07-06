use std::path::Path;

use super::typeface::*;
use super::types::ExplicitFont;

/// Format a static char slice literal with ASCII-only Unicode escapes.
pub(super) fn char_slice_literal(chars: &[char]) -> String {
    let items = chars
        .iter()
        .map(|ch| char_literal(*ch))
        .collect::<Vec<_>>()
        .join(", ");
    format!("&[{items}]")
}

/// Format a byte slice literal for generated Rust source.
pub(super) fn byte_slice_literal(bytes: &[u8]) -> String {
    let items = bytes
        .iter()
        .map(|byte| format!("0x{byte:02x}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("&[{items}]")
}

/// Format a Rust string literal with ASCII-only Unicode escapes.
pub(super) fn string_literal(value: &str) -> String {
    let mut literal = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => literal.push_str("\\\""),
            '\\' => literal.push_str("\\\\"),
            '\n' => literal.push_str("\\n"),
            '\r' => literal.push_str("\\r"),
            '\t' => literal.push_str("\\t"),
            ch if ch.is_ascii_graphic() || ch == ' ' => literal.push(ch),
            ch => literal.push_str(&format!("\\u{{{:04x}}}", ch as u32)),
        }
    }
    literal.push('"');
    literal
}

/// Escape a Rust string for compact JSONL output without adding a serde dependency.
pub(super) fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }
    escaped
}

/// Return a displayable path string with stable separators for JSONL manifests.
pub(super) fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Render a typeface byte using the MTEF documentation name when known.
pub(super) fn typeface_literal(typeface: u8) -> String {
    match typeface {
        EXPLICIT_FONT_NEG_2 => "EXPLICIT_FONT_NEG_2".to_string(),
        EXPLICIT_FONT_NEG_1 => "EXPLICIT_FONT_NEG_1".to_string(),
        FN_FUNCTION => "FN_FUNCTION".to_string(),
        FN_VARIABLE => "FN_VARIABLE".to_string(),
        FN_LC_GREEK => "FN_LC_GREEK".to_string(),
        FN_UC_GREEK => "FN_UC_GREEK".to_string(),
        FN_SYMBOL => "FN_SYMBOL".to_string(),
        FN_NUMBER => "FN_NUMBER".to_string(),
        FN_MT_EXTRA => "FN_MT_EXTRA".to_string(),
        FN_TEXT_FE => "FN_TEXT_FE".to_string(),
        FN_SPACE => "FN_SPACE".to_string(),
        other => format!("0x{other:02x}"),
    }
}

/// Format a char literal with ASCII-only Unicode escapes.
pub(super) fn char_literal(ch: char) -> String {
    if ch.is_ascii_graphic() && ch != '\'' && ch != '\\' {
        format!("'{ch}'")
    } else {
        format!("'\\u{{{:04x}}}'", ch as u32)
    }
}

/// Format an optional byte literal for generated Rust source.
pub(super) fn option_byte_literal(value: Option<u8>) -> String {
    value
        .map(|byte| format!("Some(0x{byte:02x})"))
        .unwrap_or_else(|| "None".to_string())
}

/// Format an optional explicit-font family for generated Rust source.
pub(super) fn explicit_font_literal(value: Option<ExplicitFont>) -> String {
    match value {
        Some(ExplicitFont::EuclidMathOne) => "Some(ExplicitFont::EuclidMathOne)".to_string(),
        Some(ExplicitFont::EuclidMathTwo) => "Some(ExplicitFont::EuclidMathTwo)".to_string(),
        None => "None".to_string(),
    }
}
