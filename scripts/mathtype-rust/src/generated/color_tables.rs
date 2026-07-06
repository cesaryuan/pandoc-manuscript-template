/// Generated MathType color-definition bytes for named `\color{...}` wrappers.
///
/// These prefixes were derived from helper probes of the form `\color{name}{a}`.
/// They capture the COLOR_DEF record that MathType emits before the usual
/// `0x0f <selector>` color switch and the visible content.
pub(crate) const NAMED_COLOR_DEFS: &[(&str, [u8; 8])] = &[
    ("black", [0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
    ("blue", [0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe8, 0x03]),
    ("green", [0x10, 0x00, 0x00, 0x00, 0xe8, 0x03, 0x00, 0x00]),
    ("purple", [0x10, 0x00, 0x26, 0x02, 0x8c, 0x00, 0xe8, 0x03]),
    ("red", [0x10, 0x00, 0xe8, 0x03, 0x00, 0x00, 0x00, 0x00]),
];

/// Return the generated MathType COLOR_DEF bytes for one supported named color.
pub(crate) fn named_color_def(name: &str) -> Option<&'static [u8; 8]> {
    NAMED_COLOR_DEFS
        .iter()
        .find_map(|(candidate, bytes)| (*candidate == name).then_some(bytes))
}
