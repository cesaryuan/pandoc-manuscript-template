use std::env;
use std::fs;
use std::path::PathBuf;

use crate::mtef::{write_equation_native, write_mtef_with_prefs};
use crate::ole::write_compound_file;
use crate::parser::{normalize_latex, Parser};

#[derive(Debug, Eq, PartialEq)]
struct Options {
    latex: Option<String>,
    input: Option<PathBuf>,
    output: PathBuf,
    mtef_output: Option<PathBuf>,
    prefs_file: Option<PathBuf>,
}

#[derive(Debug, Eq, PartialEq)]
enum ParseAction {
    HelpRequested,
    Run(Options),
}

/// Run the command-line converter from parsed process arguments.
pub(crate) fn run() -> Result<(), String> {
    let options = match parse_args(env::args().skip(1))? {
        ParseAction::HelpRequested => {
            println!("{}", usage());
            return Ok(());
        }
        ParseAction::Run(options) => options,
    };
    let raw_latex = match (&options.latex, &options.input) {
        (Some(latex), None) => latex.clone(),
        (None, Some(path)) => fs::read_to_string(path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
        _ => return Err("pass exactly one of --latex or --input".to_string()),
    };
    let latex = normalize_latex(&raw_latex);
    let expr = Parser::new(&latex).parse()?;
    let mtef = write_mtef_with_prefs(&latex, &expr, options.prefs_file.as_deref())?;
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
fn parse_args<I>(args: I) -> Result<ParseAction, String>
where
    I: IntoIterator<Item = String>,
{
    let mut latex = None;
    let mut input = None;
    let mut output = None;
    let mut mtef_output = None;
    let mut prefs_file = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--latex" => latex = args.next(),
            "--input" => input = args.next().map(PathBuf::from),
            "--output" => output = args.next().map(PathBuf::from),
            "--mtef-output" => mtef_output = args.next().map(PathBuf::from),
            "--prefs-file" => prefs_file = args.next().map(PathBuf::from),
            "--help" | "-h" => return Ok(ParseAction::HelpRequested),
            other => return Err(format!("unknown argument: {other}\n{}", usage())),
        }
    }
    Ok(ParseAction::Run(Options {
        latex,
        input,
        output: output.ok_or_else(usage)?,
        mtef_output,
        prefs_file,
    }))
}

/// Return the command usage shown for invalid invocations.
fn usage() -> String {
    "Usage: mathtype-rust (--latex <tex> | --input <file>) --output <ole.bin> [--mtef-output <mtef.bin>] [--prefs-file <prefs.eqp>]"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{parse_args, Options, ParseAction};
    use std::path::PathBuf;

    /// Keep the CLI help path non-failing because scripts may probe `--help`.
    #[test]
    fn help_flag_short_circuits_successfully() {
        let action = parse_args(["--help".to_string()]).expect("help should parse");
        assert_eq!(action, ParseAction::HelpRequested);
    }

    /// Preserve the existing mutually-exclusive input contract for direct LaTeX invocations.
    #[test]
    fn latex_mode_still_requires_output() {
        let action = parse_args([
            "--latex".to_string(),
            "$x$".to_string(),
            "--output".to_string(),
            "out.bin".to_string(),
        ])
        .expect("latex mode should parse");
        assert_eq!(
            action,
            ParseAction::Run(Options {
                latex: Some("$x$".to_string()),
                input: None,
                output: PathBuf::from("out.bin"),
                mtef_output: None,
                prefs_file: None,
            })
        );
    }

    /// Accept a MathType `.eqp` file for per-equation fixed defs.
    #[test]
    fn prefs_file_argument_is_parsed() {
        let action = parse_args([
            "--latex".to_string(),
            "$x$".to_string(),
            "--output".to_string(),
            "out.bin".to_string(),
            "--prefs-file".to_string(),
            "size.eqp".to_string(),
        ])
        .expect("prefs-file mode should parse");
        assert_eq!(
            action,
            ParseAction::Run(Options {
                latex: Some("$x$".to_string()),
                input: None,
                output: PathBuf::from("out.bin"),
                mtef_output: None,
                prefs_file: Some(PathBuf::from("size.eqp")),
            })
        );
    }

    /// Missing required output should stay a hard argument error after the help-path refactor.
    #[test]
    fn missing_output_is_still_rejected() {
        let err = parse_args(["--latex".to_string(), "$x$".to_string()])
            .expect_err("missing output should fail");
        assert!(err.contains("Usage: mathtype-rust"));
    }
}
