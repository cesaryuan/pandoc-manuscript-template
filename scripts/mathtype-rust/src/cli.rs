use std::env;
use std::fs;
use std::path::PathBuf;

use crate::conversion::{latex_to_equation, mtef_to_latex, ole_to_latex};

#[derive(Debug, Eq, PartialEq)]
struct EncodeOptions {
    latex: Option<String>,
    input: Option<PathBuf>,
    output: PathBuf,
    mtef_output: Option<PathBuf>,
    prefs_file: Option<PathBuf>,
}

#[derive(Debug, Eq, PartialEq)]
enum DecodeInput {
    Ole(PathBuf),
    Mtef(PathBuf),
}

#[derive(Debug, Eq, PartialEq)]
struct DecodeOptions {
    input: DecodeInput,
    latex_output: Option<PathBuf>,
}

#[derive(Debug, Eq, PartialEq)]
enum Options {
    Encode(EncodeOptions),
    Decode(DecodeOptions),
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
    match options {
        Options::Encode(options) => run_encode(options),
        Options::Decode(options) => run_decode(options),
    }
}

/// Convert one LaTeX source into OLE and optional raw MTEF outputs.
fn run_encode(options: EncodeOptions) -> Result<(), String> {
    let raw_latex = match (&options.latex, &options.input) {
        (Some(latex), None) => latex.clone(),
        (None, Some(path)) => fs::read_to_string(path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
        _ => return Err("pass exactly one of --latex or --input".to_string()),
    };
    let equation = latex_to_equation(&raw_latex, options.prefs_file.as_deref())?;

    if let Some(path) = options.mtef_output {
        fs::write(&path, &equation.mtef)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    }
    fs::write(&options.output, equation.ole)
        .map_err(|err| format!("failed to write {}: {err}", options.output.display()))?;
    eprintln!(
        "[mathtype-rust] wrote {}, mtef_bytes={}, latex_chars={}",
        options.output.display(),
        equation.mtef.len(),
        equation.normalized_latex.chars().count()
    );
    Ok(())
}

/// Recover LaTeX from one raw MTEF payload or MathType OLE carrier.
fn run_decode(options: DecodeOptions) -> Result<(), String> {
    let (input_path, latex) = match &options.input {
        DecodeInput::Ole(path) => {
            let bytes = fs::read(path)
                .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
            (path, ole_to_latex(&bytes)?)
        }
        DecodeInput::Mtef(path) => {
            let bytes = fs::read(path)
                .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
            (path, mtef_to_latex(&bytes)?)
        }
    };
    let source_may_be_lossy = latex.contains('?');

    if let Some(path) = options.latex_output {
        fs::write(&path, &latex)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        eprintln!(
            "[mathtype-rust] decoded {} to {}, latex_chars={}, source_may_be_lossy={source_may_be_lossy}",
            input_path.display(),
            path.display(),
            latex.chars().count()
        );
    } else {
        println!("{latex}");
        eprintln!(
            "[mathtype-rust] decoded {}, latex_chars={}, source_may_be_lossy={source_may_be_lossy}",
            input_path.display(),
            latex.chars().count()
        );
    }
    Ok(())
}

/// Parse both forward and reverse conversion modes while rejecting mixed contracts.
fn parse_args<I>(args: I) -> Result<ParseAction, String>
where
    I: IntoIterator<Item = String>,
{
    let mut latex = None;
    let mut input = None;
    let mut output = None;
    let mut mtef_output = None;
    let mut prefs_file = None;
    let mut ole_input = None;
    let mut mtef_input = None;
    let mut latex_output = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--latex" => latex = args.next(),
            "--input" => input = args.next().map(PathBuf::from),
            "--output" => output = args.next().map(PathBuf::from),
            "--mtef-output" => mtef_output = args.next().map(PathBuf::from),
            "--prefs-file" => prefs_file = args.next().map(PathBuf::from),
            "--ole-input" => ole_input = args.next().map(PathBuf::from),
            "--mtef-input" => mtef_input = args.next().map(PathBuf::from),
            "--latex-output" => latex_output = args.next().map(PathBuf::from),
            "--help" | "-h" => return Ok(ParseAction::HelpRequested),
            other => return Err(format!("unknown argument: {other}\n{}", usage())),
        }
    }

    let reverse_input_count = usize::from(ole_input.is_some()) + usize::from(mtef_input.is_some());
    if reverse_input_count > 0 {
        if reverse_input_count != 1 {
            return Err("pass exactly one of --ole-input or --mtef-input".to_string());
        }
        if latex.is_some()
            || input.is_some()
            || output.is_some()
            || mtef_output.is_some()
            || prefs_file.is_some()
        {
            return Err(
                "reverse conversion flags cannot be mixed with LaTeX-to-MTEF flags".to_string(),
            );
        }
        let input = match (ole_input, mtef_input) {
            (Some(path), None) => DecodeInput::Ole(path),
            (None, Some(path)) => DecodeInput::Mtef(path),
            _ => unreachable!("reverse input count was validated"),
        };
        return Ok(ParseAction::Run(Options::Decode(DecodeOptions {
            input,
            latex_output,
        })));
    }

    if latex_output.is_some() {
        return Err("--latex-output requires --ole-input or --mtef-input".to_string());
    }
    Ok(ParseAction::Run(Options::Encode(EncodeOptions {
        latex,
        input,
        output: output.ok_or_else(usage)?,
        mtef_output,
        prefs_file,
    })))
}

/// Return the command usage shown for invalid invocations.
fn usage() -> String {
    concat!(
        "Usage: mathtype-rust (--latex <tex> | --input <file>) --output <ole.bin> ",
        "[--mtef-output <mtef.bin>] [--prefs-file <prefs.eqp>]\n",
        "       mathtype-rust (--ole-input <ole.bin> | --mtef-input <mtef.bin>) ",
        "[--latex-output <formula.tex>]"
    )
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::{parse_args, DecodeInput, DecodeOptions, EncodeOptions, Options, ParseAction};
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
            ParseAction::Run(Options::Encode(EncodeOptions {
                latex: Some("$x$".to_string()),
                input: None,
                output: PathBuf::from("out.bin"),
                mtef_output: None,
                prefs_file: None,
            }))
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
            ParseAction::Run(Options::Encode(EncodeOptions {
                latex: Some("$x$".to_string()),
                input: None,
                output: PathBuf::from("out.bin"),
                mtef_output: None,
                prefs_file: Some(PathBuf::from("size.eqp")),
            }))
        );
    }

    /// Parse raw-MTEF reverse conversion without requiring a file output.
    #[test]
    fn mtef_decode_mode_is_parsed() {
        let action = parse_args(["--mtef-input".to_string(), "formula.mtef.bin".to_string()])
            .expect("MTEF decode mode should parse");
        assert_eq!(
            action,
            ParseAction::Run(Options::Decode(DecodeOptions {
                input: DecodeInput::Mtef(PathBuf::from("formula.mtef.bin")),
                latex_output: None,
            }))
        );
    }

    /// Parse OLE reverse conversion with an explicit LaTeX output path.
    #[test]
    fn ole_decode_output_is_parsed() {
        let action = parse_args([
            "--ole-input".to_string(),
            "formula.ole.bin".to_string(),
            "--latex-output".to_string(),
            "formula.tex".to_string(),
        ])
        .expect("OLE decode mode should parse");
        assert_eq!(
            action,
            ParseAction::Run(Options::Decode(DecodeOptions {
                input: DecodeInput::Ole(PathBuf::from("formula.ole.bin")),
                latex_output: Some(PathBuf::from("formula.tex")),
            }))
        );
    }

    /// Missing required forward output should stay a hard argument error.
    #[test]
    fn missing_output_is_still_rejected() {
        let err = parse_args(["--latex".to_string(), "$x$".to_string()])
            .expect_err("missing output should fail");
        assert!(err.contains("Usage: mathtype-rust"));
    }

    /// Mixed forward and reverse flags must fail before any file is read.
    #[test]
    fn mixed_conversion_modes_are_rejected() {
        let err = parse_args([
            "--latex".to_string(),
            "$x$".to_string(),
            "--ole-input".to_string(),
            "formula.ole.bin".to_string(),
        ])
        .expect_err("mixed modes should fail");
        assert!(err.contains("cannot be mixed"));
    }
}
