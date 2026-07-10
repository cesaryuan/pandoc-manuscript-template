use std::env;
use std::fs;
use std::path::PathBuf;

use serde_json::json;

use crate::svg_backend::{RenderedSvg, SvgBackend, render_formula_svg};
use crate::wmf::svg_to_wmf;

const DEFAULT_FONT_SIZE_PT: f64 = 12.0;

#[derive(Debug, PartialEq)]
struct Options {
    latex: Option<String>,
    input: Option<PathBuf>,
    output: PathBuf,
    metadata_output: PathBuf,
    svg_output: Option<PathBuf>,
    svg_backend: SvgBackend,
    font_size_pt: f64,
}

#[derive(Debug, PartialEq)]
enum ParseAction {
    HelpRequested,
    Run(Options),
}

/// Parse arguments, render one formula, and write all requested artifacts.
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

    let rendered = render_formula_svg(&raw_latex, options.font_size_pt, options.svg_backend)?;
    let wmf = svg_to_wmf(&rendered.svg, rendered.width_pt, rendered.height_pt)?;
    write_parented(&options.output, &wmf)?;

    if let Some(path) = options.svg_output.as_ref() {
        write_parented(path, rendered.svg.as_bytes())?;
    }

    let json_bytes = serialize_metadata(&rendered, options.svg_backend, options.font_size_pt)?;
    write_parented(&options.metadata_output, &json_bytes)?;

    eprintln!(
        "[latex2wmf] backend={}, size={:.4}x{:.4}pt, wrote {}",
        options.svg_backend.as_str(),
        rendered.width_pt,
        rendered.height_pt,
        options.output.display()
    );
    Ok(())
}

/// Serialize the exact placement metadata written beside a generated WMF.
pub(crate) fn serialize_metadata(
    rendered: &RenderedSvg,
    svg_backend: SvgBackend,
    font_size_pt: f64,
) -> Result<Vec<u8>, String> {
    let raw_scale = 32.0;
    let metadata = json!({
        "bounds": {
            "width_pt": rendered.width_pt,
            "height_pt": rendered.height_pt
        },
        "mathtype": {
            "width_raw": (rendered.width_pt * raw_scale).round() as i64,
            "height_raw": (rendered.height_pt * raw_scale).round() as i64,
            "baseline_from_bottom_raw": (rendered.baseline_from_bottom_pt * raw_scale).round() as i64,
            "width_pt": rendered.width_pt,
            "height_pt": rendered.height_pt,
            "baseline_from_bottom_pt": rendered.baseline_from_bottom_pt,
            "horiz_pos_type": 0,
            "horiz_pos": 0
        },
        "renderer": {
            "svg_backend": svg_backend.as_str(),
            "baseline_source": rendered.baseline_source,
            "font_size_pt": font_size_pt,
            "wmf_geometry": "flattened-svg-paths"
        }
    });
    serde_json::to_vec_pretty(&metadata)
        .map_err(|err| format!("failed to serialize metadata: {err}"))
}

/// Write bytes after creating the output parent directory when necessary.
fn write_parented(path: &PathBuf, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(path, bytes).map_err(|err| format!("failed to write {}: {err}", path.display()))
}

/// Parse the intentionally small CLI used by the Python integration.
fn parse_args<I>(args: I) -> Result<ParseAction, String>
where
    I: IntoIterator<Item = String>,
{
    let mut latex = None;
    let mut input = None;
    let mut output = None;
    let mut metadata_output = None;
    let mut svg_output = None;
    let mut svg_backend = SvgBackend::Ratex;
    let mut font_size_pt = DEFAULT_FONT_SIZE_PT;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--latex" => latex = args.next(),
            "--input" => input = args.next().map(PathBuf::from),
            "--output" => output = args.next().map(PathBuf::from),
            "--metadata-output" => metadata_output = args.next().map(PathBuf::from),
            "--svg-output" => svg_output = args.next().map(PathBuf::from),
            "--svg-backend" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--svg-backend requires ratex or typst".to_string())?;
                svg_backend = SvgBackend::parse(&value)?;
            }
            "--font-size" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--font-size requires a point value".to_string())?;
                font_size_pt = value
                    .parse::<f64>()
                    .map_err(|_| format!("invalid --font-size value: {value}"))?;
                if !font_size_pt.is_finite() || font_size_pt <= 0.0 {
                    return Err("--font-size must be a positive finite point value".to_string());
                }
            }
            "--help" | "-h" => return Ok(ParseAction::HelpRequested),
            other => return Err(format!("unknown argument: {other}\n{}", usage())),
        }
    }

    if latex.is_some() == input.is_some() {
        return Err("pass exactly one of --latex or --input".to_string());
    }
    Ok(ParseAction::Run(Options {
        latex,
        input,
        output: output.ok_or_else(usage)?,
        metadata_output: metadata_output.ok_or_else(usage)?,
        svg_output,
        svg_backend,
        font_size_pt,
    }))
}

/// Return command help for direct developer use and packaging smoke tests.
fn usage() -> String {
    "Usage: latex2wmf (--latex <tex> | --input <file>) --output <preview.wmf> --metadata-output <metadata.json> [--svg-output <formula.svg>] [--svg-backend ratex|typst] [--font-size <pt>]"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{ParseAction, parse_args};
    use crate::svg_backend::SvgBackend;

    /// Preserve a non-failing help path for packaging probes.
    #[test]
    fn help_short_circuits() {
        assert_eq!(
            parse_args(["--help".to_string()]).expect("help should parse"),
            ParseAction::HelpRequested
        );
    }

    /// Accept both documented SVG backends at the CLI boundary.
    #[test]
    fn typst_backend_is_selectable() {
        let action = parse_args([
            "--latex".to_string(),
            "x".to_string(),
            "--output".to_string(),
            "x.wmf".to_string(),
            "--metadata-output".to_string(),
            "x.json".to_string(),
            "--svg-backend".to_string(),
            "typst".to_string(),
        ])
        .expect("typst CLI should parse");
        let ParseAction::Run(options) = action else {
            panic!("expected parsed options");
        };
        assert_eq!(options.svg_backend, SvgBackend::Typst);
    }
}
