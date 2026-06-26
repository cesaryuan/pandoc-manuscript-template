use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{self, Command};
use std::thread;
use std::time::{Duration, Instant};

#[path = "../cfb.rs"]
mod cfb;
#[path = "../mathtype_input.rs"]
mod mathtype_input;
#[path = "../typeface.rs"]
mod typeface;

use mathtype_input::mathtype_tex_payload;
use typeface::FN_TEXT;

/// Probe MathType's TeX input translator and print the resulting MTEF record outline.
fn main() -> Result<(), String> {
    let options = Options::parse(env::args().skip(1).collect())?;
    fs::create_dir_all(&options.work_dir)
        .map_err(|err| format!("failed to create {}: {err}", options.work_dir.display()))?;

    let source_count = [
        options.check_helper,
        options.latex.is_some(),
        options.input.is_some(),
        options.ole.is_some(),
        options.mtef.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();
    if source_count != 1 {
        return Err(
            "pass exactly one of --check-helper, --latex, --input, --ole, or --mtef".to_string(),
        );
    }

    if options.check_helper {
        let mtef = probe_latex("$x$", &options)?;
        println!("helper_check=ok");
        print_probe_report("$x$", &mtef);
        return Ok(());
    }

    if let Some(path) = &options.ole {
        let ole =
            fs::read(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
        let mtef = extract_mtef_from_ole(&ole)?;
        options.dump_mtef(&mtef)?;
        print_probe_report(&format!("ole:{}", path.display()), &mtef);
        return Ok(());
    }

    if let Some(path) = &options.mtef {
        let mtef =
            fs::read(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
        options.dump_mtef(&mtef)?;
        print_probe_report(&format!("mtef:{}", path.display()), &mtef);
        return Ok(());
    }

    let latex = if let Some(latex) = &options.latex {
        latex.clone()
    } else {
        let path = options.input.as_ref().expect("input path is present");
        fs::read_to_string(path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?
    };

    let mtef = probe_latex(&latex, &options)?;
    options.dump_mtef(&mtef)?;
    print_probe_report(&latex, &mtef);
    Ok(())
}

/// Encode one TeX formula through the helper and return a validated MTEF payload.
fn probe_latex(latex: &str, options: &Options) -> Result<Vec<u8>, String> {
    let probe_dir = options.probe_run_dir();
    fs::create_dir_all(&probe_dir)
        .map_err(|err| format!("failed to create {}: {err}", probe_dir.display()))?;
    let tex_path = probe_dir.join("probe.tex");
    let ole_path = probe_dir.join("probe.ole.bin");
    fs::write(&tex_path, mathtype_tex_payload(latex))
        .map_err(|err| format!("failed to write {}: {err}", tex_path.display()))?;
    run_mathtype_helper(
        &options.helper,
        &options.pre_verb,
        &tex_path,
        &ole_path,
        options.timeout_ms,
    )?;

    let ole = fs::read(&ole_path)
        .map_err(|err| format!("failed to read {}: {err}", ole_path.display()))?;
    extract_mtef_from_ole(&ole).map_err(|err| {
        let _ = fs::remove_file(&ole_path);
        err
    })
}

/// Extract the MathType MTEF payload from an OLE compound file.
fn extract_mtef_from_ole(ole: &[u8]) -> Result<Vec<u8>, String> {
    let equation_native = cfb::read_regular_stream(ole, "Equation Native")?;
    Ok(equation_native
        .get(28..)
        .ok_or_else(|| "Equation Native stream is shorter than the native header".to_string())?
        .to_vec())
}

struct Options {
    helper: PathBuf,
    work_dir: PathBuf,
    pre_verb: String,
    latex: Option<String>,
    input: Option<PathBuf>,
    ole: Option<PathBuf>,
    mtef: Option<PathBuf>,
    dump_mtef: Option<PathBuf>,
    timeout_ms: u64,
    check_helper: bool,
}

impl Options {
    /// Parse CLI options while keeping paths relative to the crate root.
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut helper = PathBuf::from(
            r"..\..\src\pandoc_manuscript\mathtype\ole_helper\bin\Release\net48\MathTypeOleHelper.exe",
        );
        let mut work_dir = PathBuf::from(r".pmt\probe-mathtype-tex");
        // Keep probe cache keys aligned with the no-pre-open helper path used in audits.
        let mut pre_verb = "0".to_string();
        let mut latex = None;
        let mut input = None;
        let mut ole = None;
        let mut mtef = None;
        let mut dump_mtef = None;
        let mut timeout_ms = 30_000u64;
        let mut check_helper = false;
        let mut index = 0;
        while index < args.len() {
            match args[index].as_str() {
                "--check-helper" => check_helper = true,
                "--helper" => {
                    index += 1;
                    helper = PathBuf::from(
                        args.get(index)
                            .ok_or_else(|| "--helper requires a path".to_string())?,
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
                    pre_verb = args
                        .get(index)
                        .ok_or_else(|| "--pre-verb requires an OLE verb number".to_string())?
                        .clone();
                }
                "--latex" => {
                    index += 1;
                    latex = Some(
                        args.get(index)
                            .ok_or_else(|| "--latex requires a TeX string".to_string())?
                            .clone(),
                    );
                }
                "--input" => {
                    index += 1;
                    input = Some(PathBuf::from(
                        args.get(index)
                            .ok_or_else(|| "--input requires a path".to_string())?,
                    ));
                }
                "--ole" => {
                    index += 1;
                    ole = Some(PathBuf::from(
                        args.get(index)
                            .ok_or_else(|| "--ole requires a path".to_string())?,
                    ));
                }
                "--mtef" => {
                    index += 1;
                    mtef = Some(PathBuf::from(
                        args.get(index)
                            .ok_or_else(|| "--mtef requires a path".to_string())?,
                    ));
                }
                "--dump-mtef" => {
                    index += 1;
                    dump_mtef = Some(PathBuf::from(
                        args.get(index)
                            .ok_or_else(|| "--dump-mtef requires a path".to_string())?,
                    ));
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
                other => return Err(format!("unknown option: {other}\n{}", usage())),
            }
            index += 1;
        }
        Ok(Self {
            helper,
            work_dir,
            pre_verb,
            latex,
            input,
            ole,
            mtef,
            dump_mtef,
            timeout_ms,
            check_helper,
        })
    }

    /// Return a per-process helper directory so concurrent probes do not share files.
    fn probe_run_dir(&self) -> PathBuf {
        self.work_dir.join(format!("run-{}", process::id()))
    }

    /// Optionally write the raw MTEF bytes extracted from the selected input.
    fn dump_mtef(&self, mtef: &[u8]) -> Result<(), String> {
        if let Some(path) = &self.dump_mtef {
            fs::write(path, mtef)
                .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        }
        Ok(())
    }
}

/// Return the CLI usage string for invalid probe invocations.
fn usage() -> &'static str {
    "Usage: probe_mathtype_tex (--check-helper | --latex <tex> | --input <file> | --ole <ole.bin> | --mtef <mtef.bin>) [--helper <exe>] [--work-dir <dir>] [--pre-verb <N>] [--dump-mtef <path>] [--timeout-ms <N>]"
}

/// Invoke the existing COM helper to let MathType encode one probe formula.
fn run_mathtype_helper(
    helper: &PathBuf,
    pre_verb: &str,
    tex_path: &PathBuf,
    ole_path: &PathBuf,
    timeout_ms: u64,
) -> Result<(), String> {
    let _ = fs::remove_file(ole_path);
    let mut command = Command::new(helper);
    command.args(["--method", "set-data"]);
    if helper_needs_pre_verb(pre_verb) {
        command.args(["--pre-verb", pre_verb]);
    }
    let mut child = command
        .args(["--format", "TeX Input Language", "--input"])
        .arg(tex_path)
        .args(["--output"])
        .arg(ole_path)
        .args(["--encoding", "utf16le", "--no-verb"])
        .spawn()
        .map_err(|err| format!("failed to run {}: {err}", helper.display()))?;
    let status =
        wait_with_timeout(&mut child, Duration::from_millis(timeout_ms)).map_err(|err| {
            let _ = fs::remove_file(ole_path);
            err
        })?;
    if !status.success() {
        let _ = fs::remove_file(ole_path);
        return Err(format!(
            "{} failed for {} with status {status}",
            helper.display(),
            tex_path.display()
        ));
    }
    Ok(())
}

/// Preserve the old CLI surface where --pre-verb 0 means skipping the pre-open step.
fn helper_needs_pre_verb(pre_verb: &str) -> bool {
    pre_verb != "0"
}

/// Wait for MathType's COM helper without allowing unsupported probes to hang.
fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Result<std::process::ExitStatus, String> {
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

/// Print a compact record outline and any raw TeX fallback text runs.
fn print_probe_report(latex: &str, mtef: &[u8]) {
    println!("latex={}", latex.trim());
    println!("mtef_len={}", mtef.len());
    let records = scan_records(mtef);
    let raw_text = raw_text_runs(&records);
    println!("record_count={}", records.len());
    if raw_text.is_empty() {
        println!("raw_text_runs=none");
    } else {
        println!("raw_text_runs={}", raw_text.join(" | "));
    }
    for record in records {
        println!("{}", record.describe());
    }
}

#[derive(Debug)]
enum ProbeRecord {
    Char {
        offset: usize,
        options: u8,
        typeface: u8,
        mtcode: u16,
        font_pos: Option<u8>,
    },
    Other {
        offset: usize,
        tag: u8,
        name: &'static str,
    },
}

impl ProbeRecord {
    /// Render one scanned record as a stable one-line diagnostic.
    fn describe(&self) -> String {
        match self {
            ProbeRecord::Char {
                offset,
                options,
                typeface,
                mtcode,
                font_pos,
            } => format!(
                "offset={offset:04x} tag=CHAR options=0x{options:02x} typeface=0x{typeface:02x} mtcode=0x{mtcode:04x} font_pos={}",
                font_pos
                    .map(|value| format!("0x{value:02x}"))
                    .unwrap_or_else(|| "none".to_string())
            ),
            ProbeRecord::Other { offset, tag, name } => {
                format!("offset={offset:04x} tag={name} byte=0x{tag:02x}")
            }
        }
    }
}

/// Scan record starts without trying to fully parse nested MTEF object lists.
fn scan_records(mtef: &[u8]) -> Vec<ProbeRecord> {
    let mut records = Vec::new();
    let mut index = 0;
    while index < mtef.len() {
        let tag = mtef[index];
        if tag == 0x02 && index + 4 < mtef.len() {
            let options = mtef[index + 1];
            let typeface = mtef[index + 2];
            let mtcode = u16::from_le_bytes([mtef[index + 3], mtef[index + 4]]);
            let font_pos = if (options & 0x04) != 0 {
                mtef.get(index + 5).copied()
            } else {
                None
            };
            records.push(ProbeRecord::Char {
                offset: index,
                options,
                typeface,
                mtcode,
                font_pos,
            });
            index += if font_pos.is_some() { 6 } else { 5 };
            continue;
        }
        if let Some(name) = tag_name(tag) {
            records.push(ProbeRecord::Other {
                offset: index,
                tag,
                name,
            });
        }
        index += 1;
    }
    records
}

/// Return a readable name for record tags that are useful while probing.
fn tag_name(tag: u8) -> Option<&'static str> {
    match tag {
        0x00 => Some("END"),
        0x01 => Some("LINE"),
        0x03 => Some("TMPL"),
        0x04 => Some("PILE"),
        0x05 => Some("MATRIX"),
        0x06 => Some("EMBELL"),
        0x08 => Some("FONT_STYLE_DEF"),
        0x10 => Some("COLOR_DEF"),
        0x11 => Some("FONT_DEF"),
        0x13 => Some("ENCODING_DEF"),
        _ => None,
    }
}

/// Collect consecutive raw TeX fallback CHAR records into strings.
fn raw_text_runs(records: &[ProbeRecord]) -> Vec<String> {
    let mut runs = Vec::new();
    let mut current = String::new();
    for record in records {
        match record {
            ProbeRecord::Char {
                options,
                typeface,
                mtcode,
                ..
            } if (*options & 0x80) != 0 && *typeface == FN_TEXT => {
                if let Some(ch) = char::from_u32(u32::from(*mtcode)) {
                    current.push(ch);
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
