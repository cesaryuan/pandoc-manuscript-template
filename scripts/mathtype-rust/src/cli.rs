use std::env;
use std::fs;
use std::path::PathBuf;

use crate::ast::Expr;
use crate::mtef::{known_environment_body_hex, write_equation_native, write_mtef};
use crate::ole::write_compound_file;
use crate::parser::{normalize_latex, Parser};

#[derive(Debug)]
struct Options {
    latex: Option<String>,
    input: Option<PathBuf>,
    output: PathBuf,
    mtef_output: Option<PathBuf>,
}

/// Run the command-line converter from parsed process arguments.
pub(crate) fn run() -> Result<(), String> {
    let options = parse_args()?;
    let raw_latex = match (&options.latex, &options.input) {
        (Some(latex), None) => latex.clone(),
        (None, Some(path)) => fs::read_to_string(path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
        _ => return Err("pass exactly one of --latex or --input".to_string()),
    };
    let latex = normalize_latex(&raw_latex);
    let mtef = if known_environment_body_hex(&latex).is_some() {
        write_mtef(&latex, &Expr::Sequence(Vec::new()))?
    } else {
        let expr = Parser::new(&latex).parse()?;
        write_mtef(&latex, &expr)?
    };
    let native = write_equation_native(&mtef)?;
    let ole_bin = write_compound_file(&native)?;

    if let Some(path) = options.mtef_output {
        fs::write(&path, &mtef)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    }
    fs::write(&options.output, ole_bin)
        .map_err(|err| format!("failed to write {}: {err}", options.output.display()))?;
    eprintln!(
        "[mathtype-rust] wrote {}, mtef_bytes={}",
        options.output.display(),
        mtef.len()
    );
    Ok(())
}

/// Parse the small CLI surface used by the sample generator and future scripts.
fn parse_args() -> Result<Options, String> {
    let mut latex = None;
    let mut input = None;
    let mut output = None;
    let mut mtef_output = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--latex" => latex = args.next(),
            "--input" => input = args.next().map(PathBuf::from),
            "--output" => output = args.next().map(PathBuf::from),
            "--mtef-output" => mtef_output = args.next().map(PathBuf::from),
            "--help" | "-h" => return Err(usage()),
            other => return Err(format!("unknown argument: {other}\n{}", usage())),
        }
    }
    Ok(Options {
        latex,
        input,
        output: output.ok_or_else(usage)?,
        mtef_output,
    })
}

/// Return the command usage shown for invalid invocations.
fn usage() -> String {
    "Usage: mathtype-rust (--latex <tex> | --input <file>) --output <ole.bin> [--mtef-output <mtef.bin>]"
        .to_string()
}
