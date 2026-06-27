use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::thread;
use std::time::{Duration, Instant};

#[path = "../cfb.rs"]
mod cfb;
#[path = "../mathtype_ansi.rs"]
mod mathtype_ansi;
#[path = "../mathtype_input.rs"]
mod mathtype_input;
#[path = "../typeface.rs"]
mod typeface;

use mathtype_ansi::encode_mathtype_text;
use mathtype_input::mathtype_tex_payload;
use typeface::FN_TEXT;

const BINARY_OPERATOR_FORMULA: &str =
    r"+ - / * ⋅ ∘ ∙ ± × ÷ ∓ ∔ ∧ ∨ ∩ ∪ ≀ ⊎ ⊓ ⊔ ⊕ ⊖ ⊗ ⊘ ⊙ ⊚ ⊛ ⊝ ◯ ∖ {}";
const RELATIONS_FORMULA: &str = r"= < > : ∈ ∋ ∝ ∼ ∽ ≂ ≃ ≅ ≈ ≊ ≍ ≎ ≏ ≐ ≑ ≒ ≓ ≖ ≗ ≜ ≡ ≤ ≥ ≦ ≧ ≫ ≬ ≳ ≷ ≺ ≻ ≼ ≽ ≾ ≿ ⊂ ⊃ ⊆ ⊇ ⊏ ⊐ ⊑ ⊒ ⊢ ⊣ ⊩ ⊪ ⊸ ⋈ ⋍ ⋐ ⋑ ⋔ ⋙ ⋛ ⋞ ⋟ ⌢ ⌣ ⩾ ⪆ ⪌ ⪕ ⪖ ⪯ ⪰ ⪷ ⪸ ⫅ ⫆ ≲ ⩽ ⪅ ≶ ⋚ ⪋ ⟂ ⊨ ⊶ ⊷";

const OVERRIDE_SPECS: &[OverrideSpec] = &[
    // Standalone double quote currently hangs in MathType TeX Input probes, so keep the known
    // body fragment stable until a reliable carrier formula is found.
    OverrideSpec::Manual {
        ch: '"',
        fragments: &[FragmentSpec::Raw(&[0x22])],
    },
    OverrideSpec::Probe {
        ch: '\u{2295}',
        formula: BINARY_OPERATOR_FORMULA,
        run_index: 3,
        extract: ExtractSpec::TrimmedRun,
    },
    OverrideSpec::Probe {
        ch: '\u{2252}',
        formula: RELATIONS_FORMULA,
        run_index: 3,
        extract: ExtractSpec::TrailingBytes(1),
    },
    OverrideSpec::Probe {
        ch: '\u{2266}',
        formula: RELATIONS_FORMULA,
        run_index: 4,
        extract: ExtractSpec::TrailingBytes(1),
    },
    OverrideSpec::Probe {
        ch: '\u{2267}',
        formula: RELATIONS_FORMULA,
        run_index: 5,
        extract: ExtractSpec::TrailingBytes(1),
    },
];

/// Generate direct-literal fallback fragments from stable MathType carrier formulas.
fn main() -> Result<(), String> {
    let config = Config::parse(env::args().skip(1).collect())?;
    fs::create_dir_all(&config.work_dir)
        .map_err(|err| format!("failed to create {}: {err}", config.work_dir.display()))?;

    let mut generated = Vec::new();
    for spec in OVERRIDE_SPECS {
        let fragments = resolve_fragments(spec, &config)?;
        generated.push((spec.ch(), fragments));
    }

    fs::write(&config.output, render_output(&generated))
        .map_err(|err| format!("failed to write {}: {err}", config.output.display()))?;
    println!("wrote {}", config.output.display());
    println!("raw_text_override_count={}", generated.len());
    Ok(())
}

struct Config {
    helper: PathBuf,
    cache_dir: PathBuf,
    helper_fingerprint: String,
    output: PathBuf,
    work_dir: PathBuf,
    pre_verb: String,
    timeout_ms: u64,
    reuse_existing: bool,
}

impl Config {
    /// Parse the small CLI surface for raw-text table generation.
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut helper = PathBuf::from(
            r"..\..\src\pandoc_manuscript\mathtype\ole_helper\bin\Release\net48\MathTypeOleHelper.exe",
        );
        let mut cache_dir = PathBuf::from(r".pmt\audit-supported-functions-mathtype\cache");
        let mut output = PathBuf::from(r"src\generated\raw_text_tables.rs");
        let mut work_dir = PathBuf::from(r".pmt\raw-text-table-generation");
        // Reuse the same helper mode as audits so generated-table probes share one cache.
        let mut pre_verb = "2".to_string();
        let mut timeout_ms = 30_000u64;
        let mut reuse_existing = true;
        let mut index = 0usize;
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
                "--cache-dir" => {
                    index += 1;
                    cache_dir = PathBuf::from(
                        args.get(index)
                            .ok_or_else(|| "--cache-dir requires a path".to_string())?,
                    );
                }
                "--work-dir" => {
                    index += 1;
                    work_dir = PathBuf::from(
                        args.get(index)
                            .ok_or_else(|| "--work-dir requires a path".to_string())?,
                    );
                }
                "--pre-verb" => {
                    index += 1;
                    pre_verb = require_pre_verb_two_arg(
                        args.get(index)
                            .ok_or_else(|| "--pre-verb requires an OLE verb number".to_string())?,
                    )?;
                }
                "--timeout-ms" => {
                    index += 1;
                    let raw = args
                        .get(index)
                        .ok_or_else(|| "--timeout-ms requires a number".to_string())?;
                    timeout_ms = raw
                        .parse::<u64>()
                        .map_err(|err| format!("invalid --timeout-ms {raw}: {err}"))?;
                }
                "--no-reuse" => reuse_existing = false,
                other => return Err(format!("unknown option: {other}\n{}", usage())),
            }
            index += 1;
        }
        let helper_fingerprint = file_fingerprint(&helper)?;
        Ok(Self {
            helper,
            cache_dir,
            helper_fingerprint,
            output,
            work_dir,
            pre_verb,
            timeout_ms,
            reuse_existing,
        })
    }
}

/// Return usage for invalid generator invocations.
fn usage() -> &'static str {
    "Usage: generate_raw_text_tables [--helper <exe>] [--cache-dir <dir>] [--output <raw_text_tables.rs>] [--work-dir <dir>] [--pre-verb 2] [--timeout-ms <N>] [--no-reuse]"
}

enum OverrideSpec {
    Manual {
        ch: char,
        fragments: &'static [FragmentSpec],
    },
    Probe {
        ch: char,
        formula: &'static str,
        run_index: usize,
        extract: ExtractSpec,
    },
}

impl OverrideSpec {
    /// Return the Unicode scalar this override is describing.
    fn ch(&self) -> char {
        match self {
            OverrideSpec::Manual { ch, .. } | OverrideSpec::Probe { ch, .. } => *ch,
        }
    }
}

#[derive(Clone, Copy)]
enum ExtractSpec {
    TrimmedRun,
    TrailingBytes(usize),
}

#[derive(Clone, Copy)]
enum FragmentSpec {
    Raw(&'static [u8]),
    Char(char),
}

/// Resolve one generated fragment list from either a manual fallback or a MathType probe.
fn resolve_fragments(spec: &OverrideSpec, config: &Config) -> Result<Vec<FragmentSpec>, String> {
    match spec {
        OverrideSpec::Manual { fragments, .. } => Ok(fragments.to_vec()),
        OverrideSpec::Probe {
            ch,
            formula,
            run_index,
            extract,
        } => {
            let current = encode_mathtype_text(&ch.to_string())?;
            let raw_prefix = probe_raw_bytes(*ch, formula, *run_index, *extract, config)?;
            build_fragments(*ch, &current, &raw_prefix)
        }
    }
}

/// Probe one carrier formula and return the raw-text bytes used for the target literal.
fn probe_raw_bytes(
    ch: char,
    formula: &str,
    run_index: usize,
    extract: ExtractSpec,
    config: &Config,
) -> Result<Vec<u8>, String> {
    let name = format!("u{:04x}", ch as u32);
    let tex_path = config.work_dir.join(format!("{name}.tex"));
    let ole_path = config.work_dir.join(format!("{name}.ole.bin"));
    let payload = mathtype_tex_payload(formula);
    if let Some(cached) = read_cached_mtef(&mathtype_cache_path(&payload, config))? {
        return extract_probe_bytes_from_mtef(ch, &cached, run_index, extract);
    }
    fs::write(&tex_path, &payload)
        .map_err(|err| format!("failed to write {}: {err}", tex_path.display()))?;
    if !config.reuse_existing || !ole_path.exists() {
        run_mathtype_helper(
            &config.helper,
            &config.pre_verb,
            &tex_path,
            &ole_path,
            config.timeout_ms,
        )?;
    }
    let mtef = match load_probe_mtef(&ole_path) {
        Ok(mtef) => mtef,
        Err(first_err) if config.reuse_existing => {
            run_mathtype_helper(
                &config.helper,
                &config.pre_verb,
                &tex_path,
                &ole_path,
                config.timeout_ms,
            )?;
            load_probe_mtef(&ole_path).map_err(|second_err| {
                format!(
                    "failed to read probe {} after refresh: first={first_err}; second={second_err}",
                    ole_path.display()
                )
            })?
        }
        Err(err) => return Err(err),
    };
    extract_probe_bytes_from_mtef(ch, &mtef, run_index, extract)
}

/// Extract one target byte sequence from a cached or freshly probed MTEF payload.
fn extract_probe_bytes_from_mtef(
    ch: char,
    mtef: &[u8],
    run_index: usize,
    extract: ExtractSpec,
) -> Result<Vec<u8>, String> {
    let runs = raw_text_runs(&scan_records(mtef));
    let run = runs.get(run_index).ok_or_else(|| {
        format!(
            "raw_text_runs[{run_index}] is missing for U+{:04X}",
            ch as u32
        )
    })?;
    let trimmed = trim_ascii_spaces(run);
    if trimmed.is_empty() {
        return Err(format!("carrier run is empty for U+{:04X}", ch as u32));
    }
    match extract {
        ExtractSpec::TrimmedRun => Ok(trimmed.to_vec()),
        ExtractSpec::TrailingBytes(len) => {
            if trimmed.len() < len {
                return Err(format!(
                    "carrier run is too short for U+{:04X}: need {len}, got {}",
                    ch as u32,
                    trimmed.len()
                ));
            }
            Ok(trimmed[trimmed.len() - len..].to_vec())
        }
    }
}

/// Load one probe OLE and extract the embedded Equation Native payload.
fn load_probe_mtef(ole_path: &Path) -> Result<Vec<u8>, String> {
    let ole = fs::read(ole_path)
        .map_err(|err| format!("failed to read {}: {err}", ole_path.display()))?;
    extract_mtef_from_ole(&ole)
}

/// Return the audit cache path for one exact MathType payload.
fn mathtype_cache_path(payload: &str, config: &Config) -> PathBuf {
    let key = cache_key(payload, config);
    config.cache_dir.join(&key[..2]).join(key).join("mtef.bin")
}

/// Build the same stable cache key used by audit_supported_functions.
fn cache_key(payload: &str, config: &Config) -> String {
    let material = format!(
        "v1\0payload={payload}\0pre_verb={}\0helper={}",
        config.pre_verb.as_str(),
        config.helper_fingerprint
    );
    fnv1a64_hex(material.as_bytes())
}

/// Load one cached MTEF payload when audit_supported_functions already probed it.
fn read_cached_mtef(path: &Path) -> Result<Option<Vec<u8>>, String> {
    match fs::read(path) {
        Ok(bytes) if !bytes.is_empty() => Ok(Some(bytes)),
        Ok(_) => Ok(None),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(format!("failed to read cache {}: {err}", path.display())),
    }
}

/// Hash one file into the same helper fingerprint format used by the audit cache.
fn file_fingerprint(path: &Path) -> Result<String, String> {
    let bytes =
        fs::read(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    Ok(fnv1a64_hex(&bytes))
}

/// Return a deterministic hex digest without adding an external hashing crate.
fn fnv1a64_hex(bytes: &[u8]) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{hash:016x}")
}

/// Build parser fragments from MathType's raw prefix and the current Windows ACP encoding.
fn build_fragments(
    ch: char,
    current: &[u8],
    raw_prefix: &[u8],
) -> Result<Vec<FragmentSpec>, String> {
    if current == raw_prefix || current.len() == raw_prefix.len() {
        return Ok(vec![FragmentSpec::Raw(Box::leak(
            raw_prefix.to_vec().into_boxed_slice(),
        ))]);
    }
    if current.starts_with(raw_prefix) {
        let tail = &current[raw_prefix.len()..];
        if !tail.is_empty() && tail.iter().all(u8::is_ascii) {
            let mut fragments = vec![FragmentSpec::Raw(Box::leak(
                raw_prefix.to_vec().into_boxed_slice(),
            ))];
            for byte in tail {
                fragments.push(FragmentSpec::Char(char::from(*byte)));
            }
            return Ok(fragments);
        }
    }
    Err(format!(
        "could not derive override fragments for U+{:04X}: current={:?}, raw_prefix={:?}",
        ch as u32, current, raw_prefix
    ))
}

/// Render the generated fragment table as a compact Rust module.
fn render_output(rows: &[(char, Vec<FragmentSpec>)]) -> String {
    let mut out = String::new();
    out.push_str("/// Generated direct-literal fallback fragments for MathType body records.\n");
    out.push_str("#[derive(Clone, Copy, Debug, Eq, PartialEq)]\n");
    out.push_str("pub(crate) enum LiteralOverrideFragment {\n");
    out.push_str("    Raw(&'static [u8]),\n");
    out.push_str("    Char(char),\n");
    out.push_str("}\n\n");
    out.push_str(
        "pub(crate) fn literal_raw_text_override(ch: char) -> Option<&'static [LiteralOverrideFragment]> {\n",
    );
    out.push_str("    match ch {\n");
    for (ch, fragments) in rows {
        out.push_str(&format!("        '\\u{{{:04x}}}' => Some(&[", *ch as u32));
        for (index, fragment) in fragments.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            match fragment {
                FragmentSpec::Raw(bytes) => {
                    out.push_str("LiteralOverrideFragment::Raw(&[");
                    for (byte_index, byte) in bytes.iter().enumerate() {
                        if byte_index > 0 {
                            out.push_str(", ");
                        }
                        out.push_str(&format!("0x{byte:02x}"));
                    }
                    out.push_str("])");
                }
                FragmentSpec::Char(ch) => {
                    out.push_str(&format!("LiteralOverrideFragment::Char({ch:?})"));
                }
            }
        }
        out.push_str("]),\n");
    }
    out.push_str("        _ => None,\n");
    out.push_str("    }\n");
    out.push_str("}\n");
    out
}

/// Extract the MTEF payload from MathType's OLE compound file.
fn extract_mtef_from_ole(ole: &[u8]) -> Result<Vec<u8>, String> {
    let equation_native = cfb::read_regular_stream(ole, "Equation Native")?;
    Ok(equation_native
        .get(28..)
        .ok_or_else(|| "Equation Native stream is shorter than the native header".to_string())?
        .to_vec())
}

enum ProbeRecord {
    Char {
        options: u8,
        typeface: u8,
        mtcode: u16,
    },
    Other,
}

/// Scan record starts without needing a full nested MTEF parser.
fn scan_records(mtef: &[u8]) -> Vec<ProbeRecord> {
    let mut records = Vec::new();
    let mut index = 0usize;
    while index < mtef.len() {
        if mtef[index] == 0x02 && index + 4 < mtef.len() {
            let options = mtef[index + 1];
            let typeface = mtef[index + 2];
            let mtcode = u16::from_le_bytes([mtef[index + 3], mtef[index + 4]]);
            let has_font_pos = (options & 0x04) != 0;
            records.push(ProbeRecord::Char {
                options,
                typeface,
                mtcode,
            });
            index += if has_font_pos { 6 } else { 5 };
            continue;
        }
        records.push(ProbeRecord::Other);
        index += 1;
    }
    records
}

/// Collect consecutive raw-text CHAR records into byte strings.
fn raw_text_runs(records: &[ProbeRecord]) -> Vec<Vec<u8>> {
    let mut runs = Vec::new();
    let mut current = Vec::new();
    for record in records {
        match record {
            ProbeRecord::Char {
                options,
                typeface,
                mtcode,
            } if (*options & 0x80) != 0 && *typeface == FN_TEXT => {
                let value = u32::from(*mtcode);
                if value <= u8::MAX as u32 {
                    current.push(value as u8);
                }
            }
            _ => {
                if !current.is_empty() {
                    runs.push(std::mem::take(&mut current));
                }
            }
        }
    }
    if !current.is_empty() {
        runs.push(current);
    }
    runs
}

/// Drop probe-added ASCII padding so carrier runs describe just the literal bytes.
fn trim_ascii_spaces(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|byte| *byte != b' ')
        .unwrap_or(bytes.len());
    let end = bytes
        .iter()
        .rposition(|byte| *byte != b' ')
        .map(|index| index + 1)
        .unwrap_or(start);
    &bytes[start..end]
}

/// Invoke the existing COM helper for one carrier formula.
fn run_mathtype_helper(
    helper: &Path,
    pre_verb: &str,
    tex_path: &Path,
    ole_path: &Path,
    timeout_ms: u64,
) -> Result<(), String> {
    let _ = fs::remove_file(ole_path);
    let mut command = Command::new(helper);
    command.args(["--method", "set-data"]);
    command.args(["--pre-verb", pre_verb]);
    let mut child = command
        .args(["--format", "TeX Input Language", "--input"])
        .arg(tex_path)
        .args(["--output"])
        .arg(ole_path)
        .args(["--encoding", "utf16le", "--no-verb"])
        .spawn()
        .map_err(|err| format!("failed to run {}: {err}", helper.display()))?;
    let status = wait_with_timeout(&mut child, Duration::from_millis(timeout_ms))?;
    if !status.success() {
        return Err(format!(
            "{} failed for {} with status {status}",
            helper.display(),
            tex_path.display()
        ));
    }
    Ok(())
}

/// Reject unsupported helper verbs so generated-table cache keys stay tied to the validated MathType path.
fn require_pre_verb_two_arg(pre_verb: &str) -> Result<String, String> {
    if pre_verb == "2" {
        Ok("2".to_string())
    } else {
        Err("--pre-verb only supports value 2.".to_string())
    }
}

/// Wait for the helper without allowing one probe to hang forever.
fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Result<ExitStatus, String> {
    let start = Instant::now();
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|err| format!("failed while waiting for helper: {err}"))?
        {
            return Ok(status);
        }
        if start.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "MathType helper timed out after {} ms",
                timeout.as_millis()
            ));
        }
        thread::sleep(Duration::from_millis(50));
    }
}
