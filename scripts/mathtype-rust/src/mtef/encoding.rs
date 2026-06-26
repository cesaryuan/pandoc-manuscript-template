use crate::generated::char_tables::{
    BigOperatorGlyph, CommandStyledChar, EncodedChar, StyledChar, BIG_OPERATOR_GLYPHS,
    COMMAND_SPECIFIC_CHARS, MATHBB_CHARS, MATHCAL_CHARS, MATHFRAK_CHARS, OPERATOR_CHARS,
    SPECIAL_CHARS,
};
use crate::typeface::FN_LC_GREEK;

/// Return MathType's generated style/font-position tuple for TeX command symbols.
pub(super) fn special_char(ch: char) -> Option<StyledChar> {
    SPECIAL_CHARS
        .iter()
        .copied()
        .find(|entry| entry.ch == ch)
        .or_else(|| derived_special_char(ch))
}

/// Return a command-specific character record when the source command matters.
pub(super) fn command_specific_char(command: &str) -> Option<CommandStyledChar> {
    COMMAND_SPECIFIC_CHARS
        .iter()
        .copied()
        .find(|entry| entry.command == command)
}

/// Return MathType's generated style/font-position tuple for ASCII operators.
pub(super) fn operator_char(ch: char) -> Option<EncodedChar> {
    encoded_char(OPERATOR_CHARS, ch)
}

/// Return one generated \mathcal mapping or a font-specific error.
pub(super) fn mathcal_char(ch: char) -> Result<EncodedChar, String> {
    math_font_char(MATHCAL_CHARS, ch, "mathcal")
}

/// Return one generated \mathscr mapping; MathType reuses the mathcal alphabet.
pub(super) fn mathscr_char(ch: char) -> Result<EncodedChar, String> {
    math_font_char(MATHCAL_CHARS, ch, "mathscr")
}

/// Return one generated \mathbb mapping or a font-specific error.
pub(super) fn mathbb_char(ch: char) -> Result<EncodedChar, String> {
    math_font_char(MATHBB_CHARS, ch, "mathbb")
}

/// Return one generated \mathfrak mapping or a font-specific error.
pub(super) fn mathfrak_char(ch: char) -> Result<EncodedChar, String> {
    math_font_char(MATHFRAK_CHARS, ch, "mathfrak")
}

/// Return a generated glyph used by MathType's big-operator templates.
pub(super) fn big_operator_glyph(name: &str) -> Result<BigOperatorGlyph, String> {
    BIG_OPERATOR_GLYPHS
        .iter()
        .copied()
        .find(|glyph| glyph.name == name)
        .ok_or_else(|| format!("missing generated big-operator glyph: {name}"))
}

/// Return a generated character entry from a table.
fn encoded_char(table: &[EncodedChar], ch: char) -> Option<EncodedChar> {
    table.iter().copied().find(|entry| entry.ch == ch)
}

/// Return special CHAR entries derived from documented MathType typeface slots.
fn derived_special_char(ch: char) -> Option<StyledChar> {
    match ch {
        // MathType helper currently times out for \omicron. The lowercase
        // Greek typeface follows Symbol font positions, where omicron is `o`.
        '\u{03bf}' => Some(StyledChar {
            ch,
            typeface: FN_LC_GREEK,
            mtcode: ch as u16,
            font_pos: Some(b'o'),
            explicit_font: None,
        }),
        _ => None,
    }
}

/// Return one generated math-font mapping or a font-specific error.
fn math_font_char(table: &[EncodedChar], ch: char, font_name: &str) -> Result<EncodedChar, String> {
    encoded_char(table, ch).ok_or_else(|| format!("unsupported {font_name} character: {ch}"))
}
