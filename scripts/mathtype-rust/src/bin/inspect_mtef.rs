//! Debug-only MTEF inspector for carrier formulas and sample truth objects.
//!
//! This binary reads either one MathType OLE object or one raw MTEF payload, scans
//! the byte stream for CHAR records, and prints both the record sequence and the
//! consecutive raw-text runs. It exists to debug direct-literal extraction bugs in
//! the generated tables without changing runtime parser or writer behavior.
//!
//! Usage:
//!   cargo run --bin inspect_mtef -- --ole samples\katex-supported-functions\mt_eq_038.ole.bin
//!   cargo run --bin inspect_mtef -- --mtef .tmp_probe\truth_099.mtef.bin

use std::env;
use std::fs;
use std::path::PathBuf;

#[path = "../cfb.rs"]
mod cfb;
#[path = "../typeface.rs"]
mod typeface;

use typeface::FN_TEXT;

#[derive(Clone, Copy, Debug)]
struct CharRecord {
    offset: usize,
    options: u8,
    typeface: u8,
    mtcode: u16,
    font_pos: Option<u8>,
}

/// Read one OLE or MTEF file and print its CHAR/raw-text structure.
fn main() -> Result<(), String> {
    let mut ole_path = None;
    let mut mtef_path = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--ole" => ole_path = args.next().map(PathBuf::from),
            "--mtef" => mtef_path = args.next().map(PathBuf::from),
            "--help" | "-h" => return Err(usage()),
            other => return Err(format!("unknown argument: {other}\n{}", usage())),
        }
    }
    let mtef = match (ole_path, mtef_path) {
        (Some(path), None) => load_mtef_from_ole(&path)?,
        (None, Some(path)) => fs::read(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
        _ => return Err("pass exactly one of --ole or --mtef".to_string()),
    };
    println!("mtef_len={}", mtef.len());
    let records = scan_records(&mtef);
    println!("char_record_count={}", records.len());
    for record in &records {
        println!(
            "char offset={} options=0x{:02x} typeface=0x{:02x} mtcode=0x{:04x} font_pos={}",
            record.offset,
            record.options,
            record.typeface,
            record.mtcode,
            record
                .font_pos
                .map(|value| format!("Some(0x{value:02x})"))
                .unwrap_or_else(|| "None".to_string())
        );
    }
    let runs = raw_text_runs(&records);
    println!("raw_text_run_count={}", runs.len());
    for (index, run) in runs.iter().enumerate() {
        println!(
            "raw_text_run[{index}] bytes={:?} text={}",
            run,
            run.iter().copied().map(char::from).collect::<String>()
        );
    }
    Ok(())
}

/// Return usage for invalid inspector invocations.
fn usage() -> String {
    "Usage: inspect_mtef (--ole <file> | --mtef <file>)".to_string()
}

/// Load the embedded Equation Native payload from one MathType OLE file.
fn load_mtef_from_ole(path: &PathBuf) -> Result<Vec<u8>, String> {
    let ole = fs::read(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let equation_native = cfb::read_regular_stream(&ole, "Equation Native")?;
    equation_native
        .get(28..)
        .ok_or_else(|| "Equation Native stream is shorter than the native header".to_string())
        .map(|bytes| bytes.to_vec())
}

/// Scan the MTEF payload for CHAR records without needing a full recursive parser.
fn scan_records(mtef: &[u8]) -> Vec<CharRecord> {
    let mut records = Vec::new();
    let mut index = 0usize;
    while index + 4 < mtef.len() {
        if mtef[index] == 0x02 {
            let options = mtef[index + 1];
            let has_font_pos = (options & 0x04) != 0;
            records.push(CharRecord {
                offset: index,
                options,
                typeface: mtef[index + 2],
                mtcode: u16::from_le_bytes([mtef[index + 3], mtef[index + 4]]),
                font_pos: if has_font_pos {
                    mtef.get(index + 5).copied()
                } else {
                    None
                },
            });
            index += if has_font_pos { 6 } else { 5 };
            continue;
        }
        index += 1;
    }
    records
}

/// Collect consecutive raw-text CHAR records into byte runs.
fn raw_text_runs(records: &[CharRecord]) -> Vec<Vec<u8>> {
    let mut runs = Vec::new();
    let mut current = Vec::new();
    for record in records {
        if (record.options & 0x80) != 0 && record.typeface == FN_TEXT {
            let value = u32::from(record.mtcode);
            if value <= u8::MAX as u32 {
                current.push(value as u8);
                continue;
            }
        }
        if !current.is_empty() {
            runs.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        runs.push(current);
    }
    runs
}
