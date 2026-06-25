use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SECTOR_SIZE: usize = 512;
const FREE_SECTOR: u32 = 0xffff_ffff;
const END_OF_CHAIN: u32 = 0xffff_fffe;
const FAT_SECTOR: u32 = 0xffff_fffd;
const MINI_SECTOR_SIZE: usize = 64;

#[derive(Clone, Copy)]
struct Target {
    name: &'static str,
    category: Category,
    formula: &'static str,
    selector: Selector,
}

#[derive(Clone, Copy)]
enum Category {
    MathCal,
    MathBb,
    Special,
    Operator,
    BigOperator,
}

#[derive(Clone, Copy)]
enum Selector {
    FontPos {
        ch: char,
        typeface: u8,
        font_pos: u8,
    },
    PlainChar {
        ch: char,
        typeface: u8,
        mtcode: u16,
    },
    BigOperator {
        name: &'static str,
    },
}

#[derive(Clone, Copy, Debug)]
struct CharRecord {
    typeface: u8,
    mtcode: u16,
    font_pos: Option<u8>,
}

#[derive(Clone, Copy)]
struct DirectoryEntry {
    start_sector: u32,
    size: u64,
}

/// Generate MathType-derived Rust tables for the writer.
fn main() -> Result<(), String> {
    let config = Config::parse(env::args().skip(1).collect())?;
    fs::create_dir_all(&config.work_dir)
        .map_err(|err| format!("failed to create {}: {err}", config.work_dir.display()))?;

    let mut rows = Vec::new();
    for target in build_targets()? {
        let ole_path = config.work_dir.join(format!("{}.ole.bin", target.name));
        let tex_path = config.work_dir.join(format!("{}.tex", target.name));
        fs::write(&tex_path, target.formula)
            .map_err(|err| format!("failed to write {}: {err}", tex_path.display()))?;
        run_mathtype_helper(&config.helper, &tex_path, &ole_path)?;
        let record = extract_target_record(&ole_path, target)?;
        rows.push((target, record));
    }

    let source = render_tables(&rows);
    if let Some(parent) = config.output.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(&config.output, source)
        .map_err(|err| format!("failed to write {}: {err}", config.output.display()))?;
    println!("wrote {}", config.output.display());
    Ok(())
}

struct Config {
    helper: PathBuf,
    output: PathBuf,
    work_dir: PathBuf,
}

impl Config {
    /// Parse generator options while keeping defaults relative to the crate root.
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut helper = PathBuf::from(
            r"..\..\src\pandoc_manuscript\mathtype\ole_helper\bin\Release\net48\MathTypeOleHelper.exe",
        );
        let mut output = PathBuf::from(r"src\generated\char_tables.rs");
        let mut work_dir = PathBuf::from(r".pmt\mtef-table-generation");

        let mut index = 0;
        while index < args.len() {
            match args[index].as_str() {
                "--helper" => {
                    index += 1;
                    helper = PathBuf::from(
                        args.get(index)
                            .ok_or_else(|| "--helper requires a path".to_string())?,
                    );
                }
                "--output" => {
                    index += 1;
                    output = PathBuf::from(
                        args.get(index)
                            .ok_or_else(|| "--output requires a path".to_string())?,
                    );
                }
                "--work-dir" => {
                    index += 1;
                    work_dir = PathBuf::from(
                        args.get(index)
                            .ok_or_else(|| "--work-dir requires a path".to_string())?,
                    );
                }
                other => return Err(format!("unknown option: {other}")),
            }
            index += 1;
        }

        Ok(Self {
            helper,
            output,
            work_dir,
        })
    }
}

/// Build every probe formula whose output is an encoding table entry.
fn build_targets() -> Result<Vec<Target>, String> {
    let mut targets = Vec::new();

    for ch in 'A'..='Z' {
        let leaked = Box::leak(format!("$\\mathcal{{{ch}}}$").into_boxed_str());
        targets.push(Target {
            name: Box::leak(format!("mathcal_{ch}").into_boxed_str()),
            category: Category::MathCal,
            formula: leaked,
            selector: Selector::FontPos {
                ch,
                typeface: 0x7f,
                font_pos: ch as u8,
            },
        });
    }

    for ch in 'A'..='Z' {
        let leaked = Box::leak(format!("$\\mathbb{{{ch}}}$").into_boxed_str());
        targets.push(Target {
            name: Box::leak(format!("mathbb_{ch}").into_boxed_str()),
            category: Category::MathBb,
            formula: leaked,
            selector: Selector::FontPos {
                ch,
                typeface: 0x7f,
                font_pos: ch as u8,
            },
        });
    }

    for &(logical, tex, typeface, font_pos, name) in special_targets() {
        targets.push(Target {
            name,
            category: Category::Special,
            formula: tex,
            selector: Selector::FontPos {
                ch: logical,
                typeface,
                font_pos,
            },
        });
    }

    for ch in ['+', '-', '=', '<', '>'] {
        let leaked = Box::leak(format!("$x{ch}y$").into_boxed_str());
        targets.push(Target {
            name: Box::leak(format!("operator_{}", operator_name(ch)).into_boxed_str()),
            category: Category::Operator,
            formula: leaked,
            selector: Selector::FontPos {
                ch,
                typeface: 0x86,
                font_pos: ch as u8,
            },
        });
    }
    targets.push(Target {
        name: "operator_asterisk",
        category: Category::Operator,
        formula: "$x*y$",
        selector: Selector::PlainChar {
            ch: '*',
            typeface: 0x82,
            mtcode: 0x002a,
        },
    });

    targets.push(Target {
        name: "bigop_sum",
        category: Category::BigOperator,
        formula: r"$\sum_{i=1}^{N}x_i$",
        selector: Selector::BigOperator { name: "sum" },
    });
    targets.push(Target {
        name: "bigop_product",
        category: Category::BigOperator,
        formula: r"$\prod_{i=1}^{N}x_i$",
        selector: Selector::BigOperator { name: "product" },
    });

    Ok(targets)
}

/// Return TeX command probes for parser-supported non-ASCII symbols.
fn special_targets() -> &'static [(char, &'static str, u8, u8, &'static str)] {
    &[
        ('\u{03b1}', r"$\alpha$", 0x84, b'a', "special_alpha"),
        ('\u{03b2}', r"$\beta$", 0x84, b'b', "special_beta"),
        ('\u{03b3}', r"$\gamma$", 0x84, b'g', "special_gamma"),
        ('\u{03b4}', r"$\delta$", 0x84, b'd', "special_delta"),
        ('\u{03bb}', r"$\lambda$", 0x84, b'l', "special_lambda"),
        ('\u{03c0}', r"$\pi$", 0x84, b'p', "special_pi"),
        ('\u{03c1}', r"$\rho$", 0x84, b'r', "special_rho"),
        ('\u{03c7}', r"$\chi$", 0x84, b'c', "special_chi"),
        ('\u{03c9}', r"$\omega$", 0x84, b'w', "special_omega"),
        ('\u{0394}', r"$\Delta$", 0x85, b'D', "special_Delta"),
        ('\u{03a8}', r"$\Psi$", 0x85, b'Y', "special_Psi"),
        ('\u{03f5}', r"$\epsilon$", 0x7f, 0xf2, "special_epsilon"),
        ('\u{00d7}', r"$\times$", 0x86, 0xb4, "special_times"),
        ('\u{22c5}', r"$\cdot$", 0x86, 0xd7, "special_cdot"),
        ('\u{2208}', r"$\in$", 0x86, 0xce, "special_in"),
        ('\u{221e}', r"$\infty$", 0x86, 0xa5, "special_infty"),
        ('\u{2190}', r"$\leftarrow$", 0x86, 0xac, "special_leftarrow"),
        ('\u{2026}', r"$\ldots$", 0x86, 0xbc, "special_ldots"),
        ('\u{2260}', r"$\ne$", 0x86, 0xb9, "special_ne"),
        ('\u{2265}', r"$\ge$", 0x86, 0xb3, "special_ge"),
    ]
}

/// Name ASCII operator probes without punctuation in file names.
fn operator_name(ch: char) -> &'static str {
    match ch {
        '+' => "plus",
        '-' => "minus",
        '=' => "equals",
        '<' => "lt",
        '>' => "gt",
        '*' => "asterisk",
        _ => "unknown",
    }
}

/// Invoke the existing COM helper to let MathType encode one probe formula.
fn run_mathtype_helper(helper: &Path, tex_path: &Path, ole_path: &Path) -> Result<(), String> {
    let status = Command::new(helper)
        .args([
            "--method",
            "set-data",
            "--pre-verb",
            "2",
            "--format",
            "TeX Input Language",
            "--input",
        ])
        .arg(tex_path)
        .args(["--output"])
        .arg(ole_path)
        .args(["--encoding", "utf16le", "--no-verb"])
        .status()
        .map_err(|err| format!("failed to run {}: {err}", helper.display()))?;
    if !status.success() {
        return Err(format!(
            "{} failed for {} with status {status}",
            helper.display(),
            tex_path.display()
        ));
    }
    Ok(())
}

/// Extract the selected CHAR record from a generated MathType OLE object.
fn extract_target_record(ole_path: &Path, target: Target) -> Result<CharRecord, String> {
    let ole = fs::read(ole_path)
        .map_err(|err| format!("failed to read {}: {err}", ole_path.display()))?;
    let equation_native = read_regular_stream(&ole, "Equation Native")?;
    let mtef = equation_native
        .get(28..)
        .ok_or_else(|| "Equation Native stream is shorter than the native header".to_string())?;
    let records = collect_char_records(mtef);
    match target.selector {
        Selector::FontPos {
            typeface, font_pos, ..
        } => match target.category {
            Category::MathCal | Category::MathBb => records
                .iter()
                .rev()
                .copied()
                .find(|record| matches!(record.typeface, 0x7e | 0x7f | 0x83 | 0x88 | 0x8b))
                .ok_or_else(|| {
                    format!(
                        "no math-font CHAR record matched {}; records={records:?}",
                        target.name
                    )
                }),
            _ => records
                .iter()
                .rev()
                .copied()
                .find(|record| record.typeface == typeface && record.font_pos == Some(font_pos))
                .ok_or_else(|| format!("no CHAR record matched {}", target.name)),
        },
        Selector::PlainChar {
            typeface, mtcode, ..
        } => records
            .iter()
            .rev()
            .copied()
            .find(|record| {
                record.typeface == typeface && record.mtcode == mtcode && record.font_pos.is_none()
            })
            .ok_or_else(|| format!("no plain CHAR record matched {}", target.name)),
        Selector::BigOperator { .. } => records
            .iter()
            .rev()
            .copied()
            .find(|record| {
                record.typeface == 0x86 && record.font_pos.is_some_and(|pos| pos >= 0x80)
            })
            .ok_or_else(|| format!("no big-operator CHAR record matched {}", target.name)),
    }
}

/// Collect explicit-font CHAR records from MTEF bytes.
fn collect_char_records(mtef: &[u8]) -> Vec<CharRecord> {
    let mut records = Vec::new();
    for index in 0..mtef.len().saturating_sub(4) {
        if mtef[index] != 0x02 {
            continue;
        }
        let options = mtef[index + 1];
        let typeface = mtef[index + 2];
        if !matches!(
            typeface,
            0x7e | 0x7f | 0x82 | 0x83 | 0x84 | 0x85 | 0x86 | 0x88 | 0x8b
        ) {
            continue;
        }
        let font_pos = if (options & 0x04) != 0 {
            mtef.get(index + 5).copied()
        } else {
            None
        };
        records.push(CharRecord {
            typeface,
            mtcode: u16::from_le_bytes([mtef[index + 3], mtef[index + 4]]),
            font_pos,
        });
    }
    records
}

/// Render all extracted rows as the generated Rust module.
fn render_tables(rows: &[(Target, CharRecord)]) -> String {
    let mut output = String::from(
        "// @generated by `cargo run --bin generate_mtef_tables`; do not edit by hand.\n\
         #[derive(Clone, Copy, Debug, Eq, PartialEq)]\n\
         pub(crate) struct EncodedChar {\n\
         \x20   pub(crate) ch: char,\n\
         \x20   pub(crate) typeface: u8,\n\
         \x20   pub(crate) mtcode: u16,\n\
         \x20   pub(crate) font_pos: Option<u8>,\n\
         }\n\
         \n\
         #[derive(Clone, Copy, Debug, Eq, PartialEq)]\n\
         pub(crate) struct StyledChar {\n\
         \x20   pub(crate) ch: char,\n\
         \x20   pub(crate) typeface: u8,\n\
         \x20   pub(crate) mtcode: u16,\n\
         \x20   pub(crate) font_pos: Option<u8>,\n\
         }\n\
         \n\
         #[derive(Clone, Copy, Debug, Eq, PartialEq)]\n\
         pub(crate) struct BigOperatorGlyph {\n\
         \x20   pub(crate) name: &'static str,\n\
         \x20   pub(crate) mtcode: u16,\n\
         \x20   pub(crate) font_pos: u8,\n\
         }\n\n",
    );

    render_encoded_table(&mut output, "MATHCAL_CHARS", rows, Category::MathCal);
    render_encoded_table(&mut output, "MATHBB_CHARS", rows, Category::MathBb);
    render_styled_table(&mut output, "SPECIAL_CHARS", rows, Category::Special);
    render_encoded_table(&mut output, "OPERATOR_CHARS", rows, Category::Operator);
    render_big_operator_table(&mut output, rows);
    output
}

/// Render a table whose rows share an implicit typeface chosen by the writer.
fn render_encoded_table(
    output: &mut String,
    const_name: &str,
    rows: &[(Target, CharRecord)],
    category: Category,
) {
    output.push_str(&format!(
        "pub(crate) const {const_name}: &[EncodedChar] = &[\n"
    ));
    for (target, record) in rows
        .iter()
        .filter(|(target, _)| target.category == category)
    {
        if let Some(ch) = selector_char(target.selector) {
            output.push_str(&format!(
                "    EncodedChar {{ ch: {}, typeface: 0x{:02x}, mtcode: 0x{:04x}, font_pos: {} }},\n",
                char_literal(ch),
                record.typeface,
                record.mtcode,
                option_byte_literal(record.font_pos)
            ));
        }
    }
    output.push_str("];\n\n");
}

/// Render symbols that carry both a MathType typeface and a font position.
fn render_styled_table(
    output: &mut String,
    const_name: &str,
    rows: &[(Target, CharRecord)],
    category: Category,
) {
    output.push_str(&format!(
        "pub(crate) const {const_name}: &[StyledChar] = &[\n"
    ));
    for (target, record) in rows
        .iter()
        .filter(|(target, _)| target.category == category)
    {
        if let Some(ch) = selector_char(target.selector) {
            output.push_str(&format!(
                "    StyledChar {{ ch: {}, typeface: 0x{:02x}, mtcode: 0x{:04x}, font_pos: {} }},\n",
                char_literal(ch),
                record.typeface,
                record.mtcode,
                option_byte_literal(record.font_pos)
            ));
        }
    }
    output.push_str("];\n\n");
}

/// Render the glyphs appended by MathType's big-operator templates.
fn render_big_operator_table(output: &mut String, rows: &[(Target, CharRecord)]) {
    output.push_str("pub(crate) const BIG_OPERATOR_GLYPHS: &[BigOperatorGlyph] = &[\n");
    for (target, record) in rows
        .iter()
        .filter(|(target, _)| target.category == Category::BigOperator)
    {
        if let Selector::BigOperator { name } = target.selector {
            output.push_str(&format!(
                "    BigOperatorGlyph {{ name: \"{name}\", mtcode: 0x{:04x}, font_pos: 0x{:02x} }},\n",
                record.mtcode,
                record.font_pos.expect("big operators must have font positions")
            ));
        }
    }
    output.push_str("];\n");
}

/// Return the logical character represented by a CHAR selector.
fn selector_char(selector: Selector) -> Option<char> {
    match selector {
        Selector::FontPos { ch, .. } | Selector::PlainChar { ch, .. } => Some(ch),
        Selector::BigOperator { .. } => None,
    }
}

/// Format a char literal with ASCII-only Unicode escapes.
fn char_literal(ch: char) -> String {
    if ch.is_ascii_graphic() && ch != '\'' && ch != '\\' {
        format!("'{ch}'")
    } else {
        format!("'\\u{{{:04x}}}'", ch as u32)
    }
}

/// Format an optional byte literal for generated Rust source.
fn option_byte_literal(value: Option<u8>) -> String {
    value
        .map(|byte| format!("Some(0x{byte:02x})"))
        .unwrap_or_else(|| "None".to_string())
}

impl PartialEq for Category {
    /// Compare categories by their discriminant.
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

/// Read a regular CFB stream from MathType's OLE file.
fn read_regular_stream(file: &[u8], stream_name: &str) -> Result<Vec<u8>, String> {
    if file.len() < SECTOR_SIZE {
        return Err("compound file is shorter than one header sector".to_string());
    }
    if file.get(..8) != Some(&[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]) {
        return Err("not a compound file binary object".to_string());
    }
    let sector_size = 1usize << read_u16_at(file, 30)?;
    if sector_size != SECTOR_SIZE {
        return Err(format!("unsupported sector size: {sector_size}"));
    }
    let first_directory_sector = read_u32_at(file, 48)?;
    let mini_stream_cutoff = read_u32_at(file, 56)? as u64;
    let first_mini_fat_sector = read_u32_at(file, 60)?;
    let mini_fat_sector_count = read_u32_at(file, 64)? as usize;
    let fat_sector_count = read_u32_at(file, 44)? as usize;
    let fat_sectors = read_header_difat(file, fat_sector_count)?;
    let fat = read_fat(file, &fat_sectors, sector_size)?;
    let directory = read_sector_chain(file, first_directory_sector, &fat, sector_size)?;
    let stream = find_directory_entry(&directory, stream_name)?;
    let root = find_directory_entry(&directory, "Root Entry")?;
    let mut data = if stream.size < mini_stream_cutoff {
        let mini_fat = read_mini_fat(
            file,
            first_mini_fat_sector,
            mini_fat_sector_count,
            &fat,
            sector_size,
        )?;
        let mini_stream = read_sector_chain(file, root.start_sector, &fat, sector_size)?;
        read_mini_stream_chain(&mini_stream, stream.start_sector, &mini_fat)?
    } else {
        read_sector_chain(file, stream.start_sector, &fat, sector_size)?
    };
    let wanted_len = usize::try_from(stream.size)
        .map_err(|_| format!("stream is too large to fit in memory: {}", stream.size))?;
    if data.len() < wanted_len {
        return Err(format!(
            "stream {stream_name} chain is shorter than declared size: {} < {wanted_len}",
            data.len()
        ));
    }
    data.truncate(wanted_len);
    Ok(data)
}

/// Decode the DIFAT entries stored directly in the CFB header.
fn read_header_difat(file: &[u8], fat_sector_count: usize) -> Result<Vec<u32>, String> {
    if fat_sector_count > 109 {
        return Err("CFB reader only supports header DIFAT entries".to_string());
    }
    let mut sectors = Vec::new();
    for index in 0..fat_sector_count {
        let sector = read_u32_at(file, 76 + index * 4)?;
        if sector != FREE_SECTOR {
            sectors.push(sector);
        }
    }
    Ok(sectors)
}

/// Load the FAT table from the listed FAT sectors.
fn read_fat(file: &[u8], fat_sectors: &[u32], sector_size: usize) -> Result<Vec<u32>, String> {
    let mut fat = Vec::new();
    for &sector in fat_sectors {
        let bytes = sector_bytes(file, sector, sector_size)?;
        for chunk in bytes.chunks_exact(4) {
            fat.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }
    }
    Ok(fat)
}

/// Follow a regular FAT chain and concatenate all sector payloads.
fn read_sector_chain(
    file: &[u8],
    start_sector: u32,
    fat: &[u32],
    sector_size: usize,
) -> Result<Vec<u8>, String> {
    if start_sector == END_OF_CHAIN {
        return Ok(Vec::new());
    }
    let mut data = Vec::new();
    let mut sector = start_sector;
    let mut guard = 0usize;
    while sector != END_OF_CHAIN {
        if sector == FREE_SECTOR || sector == FAT_SECTOR {
            return Err(format!("invalid sector in chain: 0x{sector:08x}"));
        }
        data.extend_from_slice(sector_bytes(file, sector, sector_size)?);
        sector = *fat
            .get(sector as usize)
            .ok_or_else(|| format!("sector {sector} is outside the FAT"))?;
        guard += 1;
        if guard > fat.len() {
            return Err("sector chain appears to contain a cycle".to_string());
        }
    }
    Ok(data)
}

/// Locate a named stream entry in the decoded directory bytes.
fn find_directory_entry(directory: &[u8], name: &str) -> Result<DirectoryEntry, String> {
    for entry in directory.chunks_exact(128) {
        let entry_name = directory_entry_name(entry)?;
        if entry_name == name {
            return Ok(DirectoryEntry {
                start_sector: read_u32_at(entry, 116)?,
                size: read_u64_at(entry, 120)?,
            });
        }
    }
    Err(format!("stream not found in compound file: {name}"))
}

/// Load the MiniFAT chain used for small Equation Native streams.
fn read_mini_fat(
    file: &[u8],
    first_sector: u32,
    sector_count: usize,
    fat: &[u32],
    sector_size: usize,
) -> Result<Vec<u32>, String> {
    if sector_count == 0 || first_sector == END_OF_CHAIN {
        return Ok(Vec::new());
    }
    let mut bytes = Vec::new();
    let mut sector = first_sector;
    for _ in 0..sector_count {
        bytes.extend_from_slice(sector_bytes(file, sector, sector_size)?);
        sector = *fat
            .get(sector as usize)
            .ok_or_else(|| format!("MiniFAT sector {sector} is outside the FAT"))?;
        if sector == END_OF_CHAIN {
            break;
        }
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect())
}

/// Follow a MiniFAT chain inside the root mini stream.
fn read_mini_stream_chain(
    mini_stream: &[u8],
    start_sector: u32,
    mini_fat: &[u32],
) -> Result<Vec<u8>, String> {
    if start_sector == END_OF_CHAIN {
        return Ok(Vec::new());
    }
    let mut data = Vec::new();
    let mut sector = start_sector;
    let mut guard = 0usize;
    while sector != END_OF_CHAIN {
        if sector == FREE_SECTOR {
            return Err("invalid free mini sector in chain".to_string());
        }
        let start = sector as usize * MINI_SECTOR_SIZE;
        let end = start + MINI_SECTOR_SIZE;
        let bytes = mini_stream
            .get(start..end)
            .ok_or_else(|| format!("mini sector {sector} is outside the mini stream"))?;
        data.extend_from_slice(bytes);
        sector = *mini_fat
            .get(sector as usize)
            .ok_or_else(|| format!("mini sector {sector} is outside the MiniFAT"))?;
        guard += 1;
        if guard > mini_fat.len() {
            return Err("mini sector chain appears to contain a cycle".to_string());
        }
    }
    Ok(data)
}

/// Return the file slice for a CFB sector.
fn sector_bytes(file: &[u8], sector: u32, sector_size: usize) -> Result<&[u8], String> {
    let start = SECTOR_SIZE + sector as usize * sector_size;
    let end = start + sector_size;
    file.get(start..end)
        .ok_or_else(|| format!("sector {sector} is outside the file"))
}

/// Decode a UTF-16 directory entry name.
fn directory_entry_name(entry: &[u8]) -> Result<String, String> {
    let name_len = read_u16_at(entry, 64)? as usize;
    if name_len < 2 || name_len > 64 {
        return Ok(String::new());
    }
    let raw = entry
        .get(..name_len - 2)
        .ok_or_else(|| "directory entry name is truncated".to_string())?;
    let utf16: Vec<u16> = raw
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    String::from_utf16(&utf16).map_err(|err| format!("invalid UTF-16 name: {err}"))
}

/// Read a little-endian u16 from a fixed offset.
fn read_u16_at(data: &[u8], offset: usize) -> Result<u16, String> {
    let bytes = data
        .get(offset..offset + 2)
        .ok_or_else(|| format!("u16 offset {offset} is outside the buffer"))?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

/// Read a little-endian u32 from a fixed offset.
fn read_u32_at(data: &[u8], offset: usize) -> Result<u32, String> {
    let bytes = data
        .get(offset..offset + 4)
        .ok_or_else(|| format!("u32 offset {offset} is outside the buffer"))?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

/// Read a little-endian u64 from a fixed offset.
fn read_u64_at(data: &[u8], offset: usize) -> Result<u64, String> {
    let bytes = data
        .get(offset..offset + 8)
        .ok_or_else(|| format!("u64 offset {offset} is outside the buffer"))?;
    Ok(u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]))
}
