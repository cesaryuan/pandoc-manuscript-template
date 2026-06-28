//! Import a small smoke-test sample set from MathJax JSON fixtures.
//!
//! This entrypoint reads one or more local JSON files copied from
//! `mathjax/MathJax-Tests/json`, extracts the ordered `tests[*].input` LaTeX
//! strings, writes them into a dedicated `samples/` subdirectory as `eq_*.tex`,
//! copies the source JSON files for provenance, and invokes the existing
//! `MathTypeOleHelper.exe` to generate matching `mt_eq_*.ole.bin` truth files.
//! The output directory is recreated on each run so stale sample/reference pairs
//! do not silently survive after the requested smoke set changes.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

#[path = "../mathtype_input.rs"]
mod mathtype_input;

use mathtype_input::mathtype_tex_payload;

/// Read MathJax JSON sources, emit sample files, and generate MathType truth OLE files.
fn main() -> Result<(), String> {
    let options = Options::parse(env::args().skip(1).collect())?;
    eprintln!(
        "import_mathjax_json_samples: sources={}, limit={}, output={}",
        options.sources.len(),
        options.limit,
        options.output_dir.display()
    );

    let loaded_suites = load_suites(&options)?;
    let selected_cases = select_cases(&loaded_suites, options.limit);
    if selected_cases.is_empty() {
        return Err(
            "no MathJax LaTeX inputs were extracted from the provided JSON files".to_string(),
        );
    }

    eprintln!(
        "import_mathjax_json_samples: selected {} formula(s) from {} suite(s)",
        selected_cases.len(),
        loaded_suites.len()
    );
    recreate_output_dir(&options.output_dir)?;
    copy_source_json_files(&loaded_suites, &options.output_dir)?;
    write_samples_and_truth(&selected_cases, &options)?;
    write_manifest(&selected_cases, &loaded_suites, &options.output_dir)?;

    eprintln!(
        "import_mathjax_json_samples: finished writing {} sample(s) under {}",
        selected_cases.len(),
        options.output_dir.display()
    );
    Ok(())
}

/// Command-line options for importing a small MathJax-based sample set.
struct Options {
    sources: Vec<PathBuf>,
    output_dir: PathBuf,
    helper: PathBuf,
    pre_verb: String,
    limit: usize,
    timeout_ms: u64,
}

impl Options {
    /// Parse CLI options while keeping defaults aligned with the existing helper workflow.
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut sources = Vec::new();
        let mut output_dir = PathBuf::from(r"samples\mathjax-json-smoke");
        let mut helper = PathBuf::from(
            r"..\..\src\pandoc_manuscript\mathtype\ole_helper\bin\Release\net48\MathTypeOleHelper.exe",
        );
        let mut pre_verb = "2".to_string();
        let mut limit = 10usize;
        let mut timeout_ms = 30_000u64;
        let mut index = 0;
        while index < args.len() {
            match args[index].as_str() {
                "--source" => {
                    index += 1;
                    sources.push(PathBuf::from(
                        args.get(index)
                            .ok_or_else(|| "--source requires a path".to_string())?,
                    ));
                }
                "--output-dir" => {
                    index += 1;
                    output_dir = PathBuf::from(
                        args.get(index)
                            .ok_or_else(|| "--output-dir requires a path".to_string())?,
                    );
                }
                "--helper" => {
                    index += 1;
                    helper = PathBuf::from(
                        args.get(index)
                            .ok_or_else(|| "--helper requires a path".to_string())?,
                    );
                }
                "--pre-verb" => {
                    index += 1;
                    pre_verb =
                        require_pre_verb_two_arg(args.get(index).ok_or_else(|| {
                            "--pre-verb requires an OLE verb number".to_string()
                        })?)?;
                }
                "--limit" => {
                    index += 1;
                    let raw = args
                        .get(index)
                        .ok_or_else(|| "--limit requires a number".to_string())?;
                    limit = raw
                        .parse::<usize>()
                        .map_err(|err| format!("invalid --limit {raw}: {err}"))?;
                    if limit == 0 {
                        return Err("--limit must be greater than zero".to_string());
                    }
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

        if sources.is_empty() {
            return Err(format!(
                "pass at least one --source <MathJax JSON path>\n{}",
                usage()
            ));
        }

        Ok(Self {
            sources,
            output_dir,
            helper,
            pre_verb,
            limit,
            timeout_ms,
        })
    }
}

/// One parsed MathJax JSON suite plus its original source path.
struct LoadedSuite {
    source_path: PathBuf,
    suite_name: String,
    cases: Vec<SelectedCase>,
}

/// One extracted LaTeX formula together with the upstream suite/test labels.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct SelectedCase {
    suite_name: String,
    test_name: String,
    raw_input: String,
}

/// Manifest for the generated smoke-test directory.
#[derive(Serialize)]
struct SampleManifest {
    sources: Vec<ManifestSource>,
    samples: Vec<ManifestSample>,
}

/// One copied JSON source recorded in the generated manifest.
#[derive(Serialize)]
struct ManifestSource {
    suite_name: String,
    source_file: String,
}

/// One generated sample recorded in the generated manifest.
#[derive(Serialize)]
struct ManifestSample {
    sample_id: String,
    suite_name: String,
    test_name: String,
    raw_input: String,
    wrapped_input: String,
    tex_file: String,
    truth_ole_file: String,
}

/// Minimal shape of a MathJax test suite JSON file.
#[derive(Deserialize)]
struct MathJaxSuite {
    name: Option<String>,
    tests: Map<String, Value>,
}

/// Minimal shape of one MathJax test case entry.
#[derive(Deserialize)]
struct MathJaxTestCase {
    input: Option<String>,
}

/// Return the CLI usage string for invalid invocations.
fn usage() -> &'static str {
    "Usage: import_mathjax_json_samples --source <json> [--source <json> ...] [--output-dir <dir>] [--limit <N>] [--helper <exe>] [--pre-verb 2] [--timeout-ms <N>]"
}

/// Load and parse all requested MathJax JSON suites in the order the user passed them.
fn load_suites(options: &Options) -> Result<Vec<LoadedSuite>, String> {
    let mut suites = Vec::new();
    for source_path in &options.sources {
        eprintln!(
            "import_mathjax_json_samples: loading {}",
            source_path.display()
        );
        let content = fs::read_to_string(source_path)
            .map_err(|err| format!("failed to read {}: {err}", source_path.display()))?;
        let suite: MathJaxSuite = serde_json::from_str(&content)
            .map_err(|err| format!("failed to parse {}: {err}", source_path.display()))?;
        let suite_name = suite.name.unwrap_or_else(|| {
            source_path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("mathjax-suite")
                .to_string()
        });
        let cases = collect_suite_cases(&suite_name, suite.tests)?;
        eprintln!(
            "import_mathjax_json_samples: suite={} extracted_cases={}",
            suite_name,
            cases.len()
        );
        suites.push(LoadedSuite {
            source_path: source_path.clone(),
            suite_name,
            cases,
        });
    }
    Ok(suites)
}

/// Extract ordered LaTeX inputs from one suite while skipping entries without an `input` field.
fn collect_suite_cases(
    suite_name: &str,
    tests: Map<String, Value>,
) -> Result<Vec<SelectedCase>, String> {
    let mut cases = Vec::new();
    for (test_name, test_value) in tests {
        let test_case: MathJaxTestCase = serde_json::from_value(test_value)
            .map_err(|err| format!("failed to parse suite={suite_name} test={test_name}: {err}"))?;
        if let Some(input) = test_case.input {
            cases.push(SelectedCase {
                suite_name: suite_name.to_string(),
                test_name,
                raw_input: input,
            });
        }
    }
    Ok(cases)
}

/// Pick the first `limit` extracted formulas across all suites while preserving suite order.
fn select_cases(loaded_suites: &[LoadedSuite], limit: usize) -> Vec<SelectedCase> {
    let mut selected = Vec::new();
    for suite in loaded_suites {
        for case in &suite.cases {
            if selected.len() == limit {
                return selected;
            }
            selected.push(case.clone());
        }
    }
    selected
}

/// Recreate the dedicated sample output directory so stale truth files cannot poison regressions.
fn recreate_output_dir(output_dir: &Path) -> Result<(), String> {
    if output_dir == Path::new(".") || output_dir == Path::new("samples") {
        return Err(format!(
            "refusing to recreate risky output directory {}",
            output_dir.display()
        ));
    }
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)
            .map_err(|err| format!("failed to remove {}: {err}", output_dir.display()))?;
    }
    fs::create_dir_all(output_dir)
        .map_err(|err| format!("failed to create {}: {err}", output_dir.display()))?;
    Ok(())
}

/// Copy the original MathJax JSON fixtures into the generated sample directory for provenance.
fn copy_source_json_files(loaded_suites: &[LoadedSuite], output_dir: &Path) -> Result<(), String> {
    let source_dir = output_dir.join("source");
    fs::create_dir_all(&source_dir)
        .map_err(|err| format!("failed to create {}: {err}", source_dir.display()))?;
    for suite in loaded_suites {
        let file_name = suite.source_path.file_name().ok_or_else(|| {
            format!(
                "source path has no file name: {}",
                suite.source_path.display()
            )
        })?;
        let destination = source_dir.join(file_name);
        fs::copy(&suite.source_path, &destination).map_err(|err| {
            format!(
                "failed to copy {} to {}: {err}",
                suite.source_path.display(),
                destination.display()
            )
        })?;
    }
    Ok(())
}

/// Write the selected samples and generate matching MathType truth OLE files with the helper.
fn write_samples_and_truth(
    selected_cases: &[SelectedCase],
    options: &Options,
) -> Result<(), String> {
    for (index, case) in selected_cases.iter().enumerate() {
        let sample_number = index + 1;
        let sample_id = format!("{sample_number:03}");
        let wrapped_input = mathtype_tex_payload(&case.raw_input);
        let tex_path = options.output_dir.join(format!("eq_{sample_id}.tex"));
        let ole_path = options
            .output_dir
            .join(format!("mt_eq_{sample_id}.ole.bin"));
        eprintln!(
            "import_mathjax_json_samples: sample={} suite={} test={}",
            sample_id, case.suite_name, case.test_name
        );
        fs::write(&tex_path, &wrapped_input)
            .map_err(|err| format!("failed to write {}: {err}", tex_path.display()))?;
        run_mathtype_helper(
            &options.helper,
            &options.pre_verb,
            &wrapped_input,
            &ole_path,
            options.timeout_ms,
        )?;
    }
    Ok(())
}

/// Write a deterministic manifest so the generated smoke set stays easy to audit.
fn write_manifest(
    selected_cases: &[SelectedCase],
    loaded_suites: &[LoadedSuite],
    output_dir: &Path,
) -> Result<(), String> {
    let manifest = SampleManifest {
        sources: loaded_suites
            .iter()
            .map(|suite| ManifestSource {
                suite_name: suite.suite_name.clone(),
                source_file: suite
                    .source_path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default()
                    .to_string(),
            })
            .collect(),
        samples: selected_cases
            .iter()
            .enumerate()
            .map(|(index, case)| {
                let sample_number = index + 1;
                let sample_id = format!("{sample_number:03}");
                let wrapped_input = mathtype_tex_payload(&case.raw_input);
                ManifestSample {
                    sample_id: sample_id.clone(),
                    suite_name: case.suite_name.clone(),
                    test_name: case.test_name.clone(),
                    raw_input: case.raw_input.clone(),
                    wrapped_input,
                    tex_file: format!("eq_{sample_id}.tex"),
                    truth_ole_file: format!("mt_eq_{sample_id}.ole.bin"),
                }
            })
            .collect(),
    };
    let manifest_path = output_dir.join("manifest.json");
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|err| format!("failed to serialize {}: {err}", manifest_path.display()))?;
    fs::write(&manifest_path, manifest_bytes)
        .map_err(|err| format!("failed to write {}: {err}", manifest_path.display()))?;
    Ok(())
}

/// Invoke the existing COM helper to let MathType encode one sample formula.
fn run_mathtype_helper(
    helper: &Path,
    pre_verb: &str,
    tex_payload: &str,
    ole_path: &Path,
    timeout_ms: u64,
) -> Result<(), String> {
    let _ = fs::remove_file(ole_path);
    let mut command = Command::new(helper);
    command.args(["--method", "set-data"]);
    command.args(["--pre-verb", pre_verb]);
    let mut child = command
        .args(["--format", "TeX Input Language", "--input"])
        .arg(tex_payload)
        .args(["--output"])
        .arg(ole_path)
        .args(["--encoding", "utf16le", "--no-verb"])
        .spawn()
        .map_err(|err| format!("failed to run {}: {err}", helper.display()))?;
    let status =
        wait_with_timeout(&mut child, Duration::from_millis(timeout_ms)).inspect_err(|_err| {
            let _ = fs::remove_file(ole_path);
        })?;
    if !status.success() {
        let _ = fs::remove_file(ole_path);
        return Err(format!(
            "{} failed for {} with status {status}",
            helper.display(),
            tex_payload
        ));
    }
    Ok(())
}

/// Reject unsupported helper verbs so generated truth files always use the validated path.
fn require_pre_verb_two_arg(pre_verb: &str) -> Result<String, String> {
    if pre_verb == "2" {
        Ok("2".to_string())
    } else {
        Err("--pre-verb only supports value 2.".to_string())
    }
}

/// Wait for MathType's COM helper and kill it if one sample hangs.
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

#[cfg(test)]
mod tests {
    use super::{collect_suite_cases, select_cases, LoadedSuite};
    use serde_json::{json, Map, Value};

    /// Keep suite extraction aligned with the JSON insertion order used by MathJax fixtures.
    #[test]
    fn collect_suite_cases_preserves_fixture_order() {
        let mut tests = Map::new();
        tests.insert("First".to_string(), json!({ "input": "x" }));
        tests.insert("Second".to_string(), json!({ "input": "y" }));
        tests.insert("Skip".to_string(), Value::Object(Map::new()));

        let cases = collect_suite_cases("ParserBaseTest", tests).expect("fixture parses");
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].test_name, "First");
        assert_eq!(cases[0].raw_input, "x");
        assert_eq!(cases[1].test_name, "Second");
        assert_eq!(cases[1].raw_input, "y");
    }

    /// Keep the smoke importer's cross-suite limit stable when multiple JSON files are provided.
    #[test]
    fn select_cases_stops_at_requested_limit() {
        let suites = vec![
            LoadedSuite {
                source_path: "ParserDigitsTest.json".into(),
                suite_name: "ParserDigitsTest".to_string(),
                cases: vec![
                    super::SelectedCase {
                        suite_name: "ParserDigitsTest".to_string(),
                        test_name: "Integer".to_string(),
                        raw_input: "2".to_string(),
                    },
                    super::SelectedCase {
                        suite_name: "ParserDigitsTest".to_string(),
                        test_name: "Number".to_string(),
                        raw_input: "3.14".to_string(),
                    },
                ],
            },
            LoadedSuite {
                source_path: "ParserBaseTest.json".into(),
                suite_name: "ParserBaseTest".to_string(),
                cases: vec![super::SelectedCase {
                    suite_name: "ParserBaseTest".to_string(),
                    test_name: "Identifier".to_string(),
                    raw_input: "x".to_string(),
                }],
            },
        ];

        let selected = select_cases(&suites, 2);
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].test_name, "Integer");
        assert_eq!(selected[1].test_name, "Number");
    }
}
