#![allow(dead_code)]

// This debug-only binary parses one LaTeX input with the same normalize/parse path used by the
// converter and prints the normalized source plus the full AST. It exists to inspect environment
// nesting and wrapper shapes while reconciling Rust MTEF output against MathType byte-for-byte.
// Usage:
//   cargo run --bin inspect_expr -- --input samples\generated\eq_033.tex
//   cargo run --bin inspect_expr -- --latex "\\begin{split}...\\end{split}"

use std::env;
use std::fs;
use std::path::PathBuf;

#[path = "../ast.rs"]
mod ast;
#[path = "../generated/mod.rs"]
mod generated;
#[path = "../mathtype_ansi.rs"]
mod mathtype_ansi;
#[path = "../parser.rs"]
mod parser;
#[path = "../raw_fallback.rs"]
mod raw_fallback;
#[path = "../typeface.rs"]
mod typeface;

use parser::{normalize_latex, Parser};

/// Parse one formula and print the normalized source and AST for mismatch debugging.
fn main() -> Result<(), String> {
    let mut latex = None;
    let mut input = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--latex" => latex = args.next(),
            "--input" => input = args.next().map(PathBuf::from),
            "--help" | "-h" => return Err(usage()),
            other => return Err(format!("unknown argument: {other}\n{}", usage())),
        }
    }
    let raw = match (latex, input) {
        (Some(latex), None) => latex,
        (None, Some(path)) => fs::read_to_string(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
        _ => return Err("pass exactly one of --latex or --input".to_string()),
    };
    let normalized = normalize_latex(&raw);
    let expr = Parser::new(&normalized).parse()?;
    println!("normalized={normalized}");
    println!("ast={expr:#?}");
    Ok(())
}

/// Return usage for invalid inspect_expr invocations.
fn usage() -> String {
    "Usage: inspect_expr (--latex <tex> | --input <file>)".to_string()
}
