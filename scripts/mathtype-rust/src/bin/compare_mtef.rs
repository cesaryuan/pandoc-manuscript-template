//! Debug-only byte comparator for MathType OLE objects and raw MTEF payloads.
//!
//! This binary loads two inputs, normalizes either side to the embedded MTEF
//! payload, finds the first byte difference, and prints a compact hex dump
//! around that offset. It exists only to speed up mismatch forensics while we
//! reconcile Rust output against MathType sample truth without changing library
//! behavior.
//!
//! Usage:
//!   cargo run --bin compare_mtef -- --left-ole samples\mathjax-json-full\mt_eq_848.ole.bin --right-ole .tmp_eq848.ole.bin
//!   cargo run --bin compare_mtef -- --left-mtef left.bin --right-mtef right.bin

use std::env;
use std::fs;
use std::path::PathBuf;

#[path = "../cfb.rs"]
mod cfb;

/// Distinguish the two supported compare input carriers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputKind {
    Ole,
    Mtef,
}

/// Keep the parsed compare command-line contract small and explicit.
#[derive(Debug)]
struct CompareArgs {
    left_kind: InputKind,
    left_path: PathBuf,
    right_kind: InputKind,
    right_path: PathBuf,
}

/// Load both inputs, compare their MTEF bytes, and print the first mismatch window.
fn main() -> Result<(), String> {
    let args = parse_args(env::args().skip(1))?;
    let left = load_input(args.left_kind, &args.left_path)?;
    let right = load_input(args.right_kind, &args.right_path)?;
    let first_diff = left
        .iter()
        .zip(right.iter())
        .position(|(lhs, rhs)| lhs != rhs)
        .unwrap_or_else(|| left.len().min(right.len()));
    println!("left_len={}", left.len());
    println!("right_len={}", right.len());
    println!("first_diff={first_diff}");
    print_window("left", &left, first_diff);
    print_window("right", &right, first_diff);
    Ok(())
}

/// Parse the tiny debug CLI while rejecting ambiguous mixed input modes.
fn parse_args<I>(args: I) -> Result<CompareArgs, String>
where
    I: IntoIterator<Item = String>,
{
    let mut left = None;
    let mut right = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--left-ole" => {
                left = args
                    .next()
                    .map(|path| (InputKind::Ole, PathBuf::from(path)))
            }
            "--left-mtef" => {
                left = args
                    .next()
                    .map(|path| (InputKind::Mtef, PathBuf::from(path)))
            }
            "--right-ole" => {
                right = args
                    .next()
                    .map(|path| (InputKind::Ole, PathBuf::from(path)))
            }
            "--right-mtef" => {
                right = args
                    .next()
                    .map(|path| (InputKind::Mtef, PathBuf::from(path)))
            }
            "--help" | "-h" => return Err(usage()),
            other => return Err(format!("unknown argument: {other}\n{}", usage())),
        }
    }
    let (left_kind, left_path) =
        left.ok_or_else(|| "missing one left input flag\n".to_string() + &usage())?;
    let (right_kind, right_path) =
        right.ok_or_else(|| "missing one right input flag\n".to_string() + &usage())?;
    Ok(CompareArgs {
        left_kind,
        left_path,
        right_kind,
        right_path,
    })
}

/// Return the usage string shown for invalid invocations.
fn usage() -> String {
    "Usage: compare_mtef (--left-ole <file> | --left-mtef <file>) (--right-ole <file> | --right-mtef <file>)".to_string()
}

/// Load one input and normalize it to the raw MTEF payload.
fn load_input(kind: InputKind, path: &PathBuf) -> Result<Vec<u8>, String> {
    match kind {
        InputKind::Ole => load_mtef_from_ole(path),
        InputKind::Mtef => {
            fs::read(path).map_err(|err| format!("failed to read {}: {err}", path.display()))
        }
    }
}

/// Extract the Equation Native payload from one MathType compound file.
fn load_mtef_from_ole(path: &PathBuf) -> Result<Vec<u8>, String> {
    let ole = fs::read(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let equation_native = cfb::read_regular_stream(&ole, "Equation Native")?;
    equation_native
        .get(28..)
        .ok_or_else(|| "Equation Native stream is shorter than the native header".to_string())
        .map(|bytes| bytes.to_vec())
}

/// Print a compact hex window around the first differing offset.
fn print_window(label: &str, bytes: &[u8], center: usize) {
    let start = center.saturating_sub(16);
    let end = (center + 16).min(bytes.len());
    let dump = bytes[start..end]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ");
    println!("{label}_window_start={start}");
    println!("{label}_window={dump}");
}
