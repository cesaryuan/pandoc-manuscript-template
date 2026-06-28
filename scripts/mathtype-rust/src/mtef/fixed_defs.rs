use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Keep the legacy fixed defs as a byte-for-byte fallback outside the source tree.
const LEGACY_FIXED_DEFS: &[u8] = &[
    0x13, b'W', b'i', b'n', b'A', b'l', b'l', b'B', b'a', b's', b'i', b'c', b'C', b'o', b'd', b'e',
    b'P', b'a', b'g', b'e', b's', 0x00, 0x11, 0x05, b'T', b'i', b'm', b'e', b's', b' ', b'N', b'e',
    b'w', b' ', b'R', b'o', b'm', b'a', b'n', 0x00, 0x11, 0x03, b'S', b'y', b'm', b'b', b'o', b'l',
    0x00, 0x11, 0x05, b'C', b'o', b'u', b'r', b'i', b'e', b'r', b' ', b'N', b'e', b'w', 0x00, 0x11,
    0x04, b'M', b'T', b' ', b'E', b'x', b't', b'r', b'a', 0x00, 0x13, b'W', b'i', b'n', b'A', b'l',
    b'l', b'C', b'o', b'd', b'e', b'P', b'a', b'g', b'e', b's', 0x00, 0x11, 0x06, 0xcb, 0xce, 0xcc,
    0xe5, 0x00, 0x12, 0x00, 0x08, 0x21, 0x2f, 0x45, 0x8f, 0x44, 0x2f, 0x41, 0x50, 0xf4, 0x10, 0x0f,
    0x47, 0x5f, 0x41, 0x50, 0xf2, 0x1f, 0x1e, 0x41, 0x50, 0xf4, 0x15, 0x0f, 0x41, 0x00, 0xf4, 0x45,
    0xf4, 0x25, 0xf4, 0x8f, 0x42, 0x5f, 0x41, 0x00, 0xf4, 0x10, 0x0f, 0x43, 0x5f, 0x41, 0x00, 0xf4,
    0x8f, 0x45, 0xf4, 0x2a, 0x5f, 0x48, 0xf4, 0x8f, 0x41, 0x00, 0xf4, 0x10, 0x0f, 0x40, 0xf4, 0x8f,
    0x41, 0x7f, 0x48, 0xf4, 0x10, 0x0f, 0x41, 0x2a, 0x5f, 0x44, 0x5f, 0x45, 0xf4, 0x5f, 0x45, 0xf4,
    0x5f, 0x41, 0x0f, 0x0c, 0x01, 0x00, 0x01, 0x00, 0x01, 0x02, 0x02, 0x02, 0x02, 0x00, 0x02, 0x00,
    0x01, 0x01, 0x01, 0x00, 0x03, 0x00, 0x01, 0x00, 0x04, 0x00, 0x05, 0x00,
];

/// Resolve the shipped MathType prefs file from the repo root when available.
const DEFAULT_PREFS_RELATIVE_PATH: &str =
    "../../src/pandoc_manuscript/mathtype/Times+Symbol 12.eqp";

/// Match MathType's default non-FE encoding name from the reference stream.
const WIN_ALL_BASIC_CODE_PAGES: &[u8] = b"WinAllBasicCodePages";

/// Match MathType's FE encoding name from the reference stream.
const WIN_ALL_CODE_PAGES: &[u8] = b"WinAllCodePages";

/// Preserve the legacy Songti fallback bytes without introducing a new codec dependency.
const SONGTI_BYTES: &[u8] = &[0xcb, 0xce, 0xcc, 0xe5];

/// Keep the Style section in MathType's fnTEXT..fnTEXT_FE order.
const STYLE_KEYS: [&str; 12] = [
    "Text", "Function", "Variable", "LCGreek", "UCGreek", "Symbol", "Vector", "Number", "User1",
    "User2", "MTExtra", "TextFE",
];

/// Keep the Size section in MathType's EQN_PREFS size-array order.
const SIZE_KEYS: [&str; 8] = [
    "Full",
    "Script",
    "ScriptScript",
    "Symbol",
    "SubSymbol",
    "User1",
    "User2",
    "SmallLargeIncr",
];

/// Keep the Spacing section in MathType's EQN_PREFS spacing-array order.
const SPACING_KEYS: [(&str, &[&str]); 30] = [
    ("LineSpacing", &["LineSpacing"]),
    ("MatrixRowSpacing", &["MatrixRowSpacing"]),
    ("MatrixColSpacing", &["MatrixColSpacing"]),
    ("SuperscriptHeight", &["SuperscriptHeight"]),
    ("SubscriptDepth", &["SubscriptDepth"]),
    ("SubSupGap", &["SubSupGap"]),
    ("LimHeight", &["LimHeight"]),
    ("LimDepth", &["LimDepth"]),
    ("LimLineSpacing", &["LimLineSpacing"]),
    ("NumerHeight", &["NumerHeight"]),
    ("DenomDepth", &["DenomDepth"]),
    ("FractBarOver", &["FractBarOver"]),
    ("FractBarThick", &["FractBarThick"]),
    ("SubFractBarThick", &["SubFractBarThick"]),
    ("FractGap", &["FractGap"]),
    ("FenceOver", &["FenceOver"]),
    ("OperSpacing", &["OperSpacing"]),
    ("NonOperSpacing", &["NonOperSpacing"]),
    ("CharWidth", &["CharWidth"]),
    ("MinGap", &["MinGap"]),
    ("VertRadGap", &["VertRadGap"]),
    ("HorizRadGap", &["HorizRadGap"]),
    ("RadWidth", &["RadWidth"]),
    ("EmbellGap", &["EmbellGap"]),
    ("PrimeHeight", &["PrimeHeight"]),
    ("BoxStrokeThick", &["BoxStrokeThick"]),
    // MathType's shipped template spells this key without the first 'r'.
    ("StrikeThruThick", &["StrikeThruThick", "StikeThruThick"]),
    ("MatrixLineThick", &["MatrixLineThick"]),
    ("RadStrokeThick", &["RadStrokeThick"]),
    ("HorizFenceGap", &["HorizFenceGap"]),
];

/// Cache the fixed defs so repeated conversions do not keep reparsing the same prefs file.
static FIXED_DEFS: OnceLock<Result<Vec<u8>, String>> = OnceLock::new();

/// Return the fixed definition block used ahead of every equation body.
pub(super) fn fixed_defs() -> Result<&'static [u8], String> {
    match FIXED_DEFS.get_or_init(build_fixed_defs_once) {
        Ok(bytes) => Ok(bytes.as_slice()),
        Err(err) => Err(err.clone()),
    }
}

/// Build fixed defs from an explicit prefs file for per-equation size overrides.
pub(super) fn fixed_defs_from_prefs_file(path: &Path) -> Result<Vec<u8>, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    build_fixed_defs_from_eqp_text(&text)
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

/// Build fixed defs from the source-tree prefs file when it is available.
fn build_fixed_defs_once() -> Result<Vec<u8>, String> {
    let prefs_path = source_prefs_path();
    if !prefs_path.exists() {
        return Ok(LEGACY_FIXED_DEFS.to_vec());
    }
    let text = fs::read_to_string(&prefs_path)
        .map_err(|err| format!("failed to read {}: {err}", prefs_path.display()))?;
    build_fixed_defs_from_eqp_text(&text)
        .map_err(|err| format!("failed to parse {}: {err}", prefs_path.display()))
}

/// Resolve the repo-local MathType prefs file relative to this crate's manifest.
fn source_prefs_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_PREFS_RELATIVE_PATH)
}

/// Parse one MathType `.eqp` file and emit the matching fixed defs records.
fn build_fixed_defs_from_eqp_text(text: &str) -> Result<Vec<u8>, String> {
    let eqp = parse_eqp(text)?;
    let sizes = SIZE_KEYS
        .iter()
        .map(|key| parse_dimension(required_value(&eqp.sizes, key, &[*key])?))
        .collect::<Result<Vec<_>, _>>()?;
    let spacing = SPACING_KEYS
        .iter()
        .map(|(label, aliases)| parse_dimension(required_value(&eqp.spacing, label, aliases)?))
        .collect::<Result<Vec<_>, _>>()?;

    let mut basic_font_defs = Vec::new();
    let mut basic_font_lookup = HashMap::new();
    let mut style_defs = Vec::with_capacity(STYLE_KEYS.len());
    for key in STYLE_KEYS.iter().take(11) {
        let style = parse_style_value(required_value(&eqp.styles, key, &[*key])?)?;
        let font_name = encode_font_name(&style.font_name)?;
        let encoding_index = basic_encoding_index_for_font(&style.font_name);
        let font_def_index = register_font_def(
            &mut basic_font_defs,
            &mut basic_font_lookup,
            encoding_index,
            font_name,
        );
        style_defs.push(Some((font_def_index, style.style_bits)));
    }

    let text_fe_style = if let Some(raw) = eqp.styles.get("TextFE") {
        let style = parse_style_value(raw)?;
        Some((
            encode_text_fe_font_name(&style.font_name)?,
            style.style_bits,
        ))
    } else {
        None
    };

    let mut out = Vec::new();
    write_encoding_def(WIN_ALL_BASIC_CODE_PAGES, &mut out)?;
    for (encoding_index, font_name) in &basic_font_defs {
        write_font_def(*encoding_index, font_name, &mut out)?;
    }

    write_encoding_def(WIN_ALL_CODE_PAGES, &mut out)?;
    let text_fe_font_index = u8::try_from(basic_font_defs.len() + 1)
        .map_err(|_| "too many FONT_DEF records".to_string())?;
    match &text_fe_style {
        Some((font_name, _)) => write_font_def(6, font_name, &mut out)?,
        None => write_font_def(6, SONGTI_BYTES, &mut out)?,
    }
    style_defs.push(Some((
        text_fe_font_index,
        text_fe_style.map(|(_, style_bits)| style_bits).unwrap_or(0),
    )));

    out.push(0x12);
    out.push(0x00);
    out.push(u8::try_from(sizes.len()).map_err(|_| "too many size values".to_string())?);
    pack_dimension_array(&sizes, &mut out)?;
    out.push(u8::try_from(spacing.len()).map_err(|_| "too many spacing values".to_string())?);
    pack_dimension_array(&spacing, &mut out)?;
    out.push(u8::try_from(style_defs.len()).map_err(|_| "too many style values".to_string())?);
    for style_def in style_defs {
        if let Some((font_index, style_bits)) = style_def {
            write_unsigned(usize::from(font_index), &mut out)?;
            out.push(style_bits);
        } else {
            out.push(0x00);
        }
    }
    Ok(out)
}

/// Read the minimal `[Styles]`, `[Sizes]`, and `[Spacing]` sections from a MathType `.eqp`.
fn parse_eqp(text: &str) -> Result<ParsedEqp, String> {
    let mut eqp = ParsedEqp::default();
    let mut current_section = Section::Other;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        if let Some(section_name) = line
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        {
            current_section = Section::from_name(section_name);
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("invalid key/value line: {line}"))?;
        let target = match current_section {
            Section::Styles => &mut eqp.styles,
            Section::Sizes => &mut eqp.sizes,
            Section::Spacing => &mut eqp.spacing,
            Section::Other => continue,
        };
        target.insert(key.trim().to_string(), value.trim().to_string());
    }
    Ok(eqp)
}

/// Return one required `.eqp` value, accepting a small alias list for known template quirks.
fn required_value<'a>(
    values: &'a HashMap<String, String>,
    label: &str,
    aliases: &[&str],
) -> Result<&'a str, String> {
    for alias in aliases {
        if let Some(value) = values.get(*alias) {
            return Ok(value.as_str());
        }
    }
    Err(format!("missing `{label}` in MathType prefs"))
}

/// Parse one style entry such as `Times New Roman,I` or `MT Extra`.
fn parse_style_value(raw: &str) -> Result<ParsedStyle, String> {
    let trimmed = raw.trim();
    if let Some((font_name, raw_flags)) = trimmed.rsplit_once(',') {
        let style_bits = parse_style_bits(raw_flags)?;
        Ok(ParsedStyle {
            font_name: font_name.trim().to_string(),
            style_bits,
        })
    } else {
        Ok(ParsedStyle {
            font_name: trimmed.to_string(),
            style_bits: 0,
        })
    }
}

/// Convert MathType's `B`/`I` suffixes into the MTEF character-style bits.
fn parse_style_bits(raw_flags: &str) -> Result<u8, String> {
    let mut style_bits = 0u8;
    for flag in raw_flags.chars().filter(|ch| !ch.is_whitespace()) {
        match flag {
            'B' | 'b' => style_bits |= 0x01,
            'I' | 'i' => style_bits |= 0x02,
            _ => return Err(format!("unsupported style flag `{flag}` in `{raw_flags}`")),
        }
    }
    Ok(style_bits)
}

/// Parse one `.eqp` size or spacing value into the MTEF dimension-array form.
fn parse_dimension(raw: &str) -> Result<Dimension, String> {
    let trimmed = raw.trim();
    let lower = trimmed.to_ascii_lowercase();
    for (suffix, unit) in [
        ("points", Unit::Points),
        ("point", Unit::Points),
        ("pt", Unit::Points),
        ("picas", Unit::Picas),
        ("pica", Unit::Picas),
        ("pc", Unit::Picas),
        ("centimeters", Unit::Centimeters),
        ("centimeter", Unit::Centimeters),
        ("cm", Unit::Centimeters),
        ("inches", Unit::Inches),
        ("inch", Unit::Inches),
        ("in", Unit::Inches),
        ("%", Unit::Percent),
    ] {
        if lower.ends_with(suffix) {
            let value = trimmed[..trimmed.len() - suffix.len()].trim();
            validate_dimension_value(value)?;
            return Ok(Dimension {
                unit,
                value: value.to_string(),
            });
        }
    }
    Err(format!("unsupported dimension `{raw}`"))
}

/// Reject dimension strings that cannot be represented in MathType's nibble stream.
fn validate_dimension_value(value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err("dimension value is empty".to_string());
    }
    for ch in value.chars() {
        if !matches!(ch, '0'..='9' | '.' | '-') {
            return Err(format!(
                "unsupported dimension character `{ch}` in `{value}`"
            ));
        }
    }
    Ok(())
}

/// Pack one dimension array exactly the way EQN_PREFS stores sizes and spacing values.
fn pack_dimension_array(values: &[Dimension], out: &mut Vec<u8>) -> Result<(), String> {
    let mut nibbles = Vec::new();
    for value in values {
        nibbles.push(value.unit.as_nibble());
        for ch in value.value.chars() {
            nibbles.push(dimension_value_nibble(ch)?);
        }
        nibbles.push(0x0f);
    }
    for pair in nibbles.chunks(2) {
        let upper = pair[0] << 4;
        let lower = pair.get(1).copied().unwrap_or(0);
        out.push(upper | lower);
    }
    Ok(())
}

/// Convert one dimension character into MathType's packed nibble alphabet.
fn dimension_value_nibble(ch: char) -> Result<u8, String> {
    match ch {
        '0'..='9' => Ok(ch as u8 - b'0'),
        '.' => Ok(0x0a),
        '-' => Ok(0x0b),
        _ => Err(format!("unsupported dimension character `{ch}`")),
    }
}

/// Pick the same predefined encoding indices MathType uses for the shipped template.
fn basic_encoding_index_for_font(font_name: &str) -> u8 {
    match font_name {
        "Symbol" => 3,
        "MT Extra" => 4,
        _ => 5,
    }
}

/// Encode one normal font name for FONT_DEF, keeping the implementation ASCII-only by default.
fn encode_font_name(font_name: &str) -> Result<Vec<u8>, String> {
    if font_name.is_ascii() {
        Ok(font_name.as_bytes().to_vec())
    } else {
        Err(format!("font name `{font_name}` is not ASCII"))
    }
}

/// Encode the optional TextFE font name or keep the legacy Songti bytes when it is requested.
fn encode_text_fe_font_name(font_name: &str) -> Result<Vec<u8>, String> {
    if font_name == "Songti" || font_name == "SimSun" {
        Ok(SONGTI_BYTES.to_vec())
    } else {
        encode_font_name(font_name)
    }
}

/// Reuse a previously emitted FONT_DEF when another style points to the same font/encoding pair.
fn register_font_def(
    defs: &mut Vec<(u8, Vec<u8>)>,
    lookup: &mut HashMap<(u8, Vec<u8>), u8>,
    encoding_index: u8,
    font_name: Vec<u8>,
) -> u8 {
    if let Some(index) = lookup.get(&(encoding_index, font_name.clone())) {
        return *index;
    }
    let next_index = u8::try_from(defs.len() + 1).expect("FONT_DEF index fits in u8");
    defs.push((encoding_index, font_name.clone()));
    lookup.insert((encoding_index, font_name), next_index);
    next_index
}

/// Write one ENCODING_DEF record.
fn write_encoding_def(name: &[u8], out: &mut Vec<u8>) -> Result<(), String> {
    if name.contains(&0) {
        return Err("encoding name contains NUL".to_string());
    }
    out.push(0x13);
    out.extend_from_slice(name);
    out.push(0x00);
    Ok(())
}

/// Write one FONT_DEF record.
fn write_font_def(encoding_index: u8, name: &[u8], out: &mut Vec<u8>) -> Result<(), String> {
    if name.contains(&0) {
        return Err("font name contains NUL".to_string());
    }
    out.push(0x11);
    write_unsigned(usize::from(encoding_index), out)?;
    out.extend_from_slice(name);
    out.push(0x00);
    Ok(())
}

/// Write the small unsigned integers used by FONT_DEF and style-array references.
fn write_unsigned(value: usize, out: &mut Vec<u8>) -> Result<(), String> {
    if value < 255 {
        out.push(value as u8);
    } else if value <= u16::MAX as usize {
        out.push(255);
        out.extend_from_slice(&(value as u16).to_le_bytes());
    } else {
        return Err(format!(
            "value is too large for MTEF unsigned integer: {value}"
        ));
    }
    Ok(())
}

/// Capture the three `.eqp` sections needed to rebuild the fixed defs.
#[derive(Default)]
struct ParsedEqp {
    styles: HashMap<String, String>,
    sizes: HashMap<String, String>,
    spacing: HashMap<String, String>,
}

/// Track the current `.eqp` section while scanning the file.
enum Section {
    Styles,
    Sizes,
    Spacing,
    Other,
}

impl Section {
    /// Map one section header onto the subset of `.eqp` sections we actually consume.
    fn from_name(name: &str) -> Self {
        match name {
            "Styles" => Self::Styles,
            "Sizes" => Self::Sizes,
            "Spacing" => Self::Spacing,
            _ => Self::Other,
        }
    }
}

/// Hold one parsed style entry before it is turned into a FONT_DEF reference.
struct ParsedStyle {
    font_name: String,
    style_bits: u8,
}

/// Hold one parsed size or spacing value before nibble packing.
struct Dimension {
    unit: Unit,
    value: String,
}

/// Enumerate the dimension units supported by MathType's EQN_PREFS arrays.
enum Unit {
    Inches,
    Centimeters,
    Points,
    Picas,
    Percent,
}

impl Unit {
    /// Return the unit nibble defined by the MathType MTEF spec.
    fn as_nibble(&self) -> u8 {
        match self {
            Self::Inches => 0,
            Self::Centimeters => 1,
            Self::Points => 2,
            Self::Picas => 3,
            Self::Percent => 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{build_fixed_defs_from_eqp_text, source_prefs_path, LEGACY_FIXED_DEFS};
    use std::fs;

    /// The shipped `.eqp` should keep generating the same bytes as the previous hard-coded block.
    #[test]
    fn source_tree_eqp_matches_legacy_fixed_defs() {
        let text = fs::read_to_string(source_prefs_path()).expect("source-tree prefs should exist");
        let generated = build_fixed_defs_from_eqp_text(&text).expect("prefs should parse");
        assert_eq!(generated, LEGACY_FIXED_DEFS);
    }

    /// Changing the Full size in the prefs file should change the generated fixed defs.
    #[test]
    fn size_changes_in_eqp_change_generated_defs() {
        let text = fs::read_to_string(source_prefs_path()).expect("source-tree prefs should exist");
        let updated = text.replace("Full=12 pt", "Full=13 pt");
        let generated =
            build_fixed_defs_from_eqp_text(&updated).expect("updated prefs should parse");
        assert_ne!(generated, LEGACY_FIXED_DEFS);
    }

    /// Changing the text font should change the generated FONT_DEF bytes.
    #[test]
    fn style_changes_in_eqp_change_generated_defs() {
        let text = fs::read_to_string(source_prefs_path()).expect("source-tree prefs should exist");
        let updated = text.replace("Text=Times New Roman", "Text=Arial");
        let generated =
            build_fixed_defs_from_eqp_text(&updated).expect("updated prefs should parse");
        assert!(generated
            .windows(b"Arial\0".len())
            .any(|window| window == b"Arial\0"));
        assert_ne!(generated, LEGACY_FIXED_DEFS);
    }
}
