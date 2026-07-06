/// Generated direct-literal fallback fragments for MathType body records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LiteralOverrideFragment {
    Raw(&'static [u8]),
    Char(char),
}

pub(crate) fn literal_raw_text_override(ch: char) -> Option<&'static [LiteralOverrideFragment]> {
    match ch {
        '\u{0022}' => Some(&[LiteralOverrideFragment::Raw(&[0x22])]),
        '\u{0023}' => Some(&[LiteralOverrideFragment::Raw(&[0x23])]),
        '\u{0026}' => Some(&[LiteralOverrideFragment::Raw(&[0x26])]),
        '\u{2295}' => Some(&[LiteralOverrideFragment::Raw(&[0xa8, 0x6e])]),
        '\u{2252}' => Some(&[LiteralOverrideFragment::Raw(&[0xa8]), LiteralOverrideFragment::Char('P')]),
        '\u{2266}' => Some(&[LiteralOverrideFragment::Raw(&[0xa8]), LiteralOverrideFragment::Char('Q')]),
        '\u{2267}' => Some(&[LiteralOverrideFragment::Raw(&[0xa8]), LiteralOverrideFragment::Char('R')]),
        _ => None,
    }
}
