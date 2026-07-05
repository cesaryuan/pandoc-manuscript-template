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
    let manifest_samples = write_samples_and_truth(&selected_cases, &options)?;
    write_manifest(&loaded_suites, manifest_samples, &options.output_dir)?;

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
    source_dirs: Vec<PathBuf>,
    output_dir: PathBuf,
    helper: PathBuf,
    keep_going_on_helper_error: bool,
    pre_verb: String,
    limit: usize,
    timeout_ms: u64,
}

impl Options {
    /// Parse CLI options while keeping defaults aligned with the existing helper workflow.
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut sources = Vec::new();
        let mut source_dirs = Vec::new();
        let mut output_dir = PathBuf::from(r"samples\mathjax-json-smoke");
        let mut helper = PathBuf::from(
            r"..\..\src\pandoc_manuscript\mathtype\ole_helper\bin\Release\net48\MathTypeOleHelper.exe",
        );
        let mut keep_going_on_helper_error = false;
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
                "--source-dir" => {
                    index += 1;
                    source_dirs.push(PathBuf::from(
                        args.get(index)
                            .ok_or_else(|| "--source-dir requires a path".to_string())?,
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
                "--keep-going-on-helper-error" => keep_going_on_helper_error = true,
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

        if sources.is_empty() && source_dirs.is_empty() {
            return Err(format!(
                "pass at least one --source <MathJax JSON path> or --source-dir <dir>\n{}",
                usage()
            ));
        }

        Ok(Self {
            sources,
            source_dirs,
            output_dir,
            helper,
            keep_going_on_helper_error,
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
    case_id: String,
    sample_id: Option<String>,
    status: String,
    suite_name: String,
    test_name: String,
    raw_input: String,
    wrapped_input: String,
    all_tex_file: String,
    tex_file: Option<String>,
    truth_ole_file: Option<String>,
    helper_error_file: Option<String>,
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
    "Usage: import_mathjax_json_samples [--source <json> ...] [--source-dir <dir> ...] [--output-dir <dir>] [--limit <N>] [--helper <exe>] [--keep-going-on-helper-error] [--pre-verb 2] [--timeout-ms <N>]"
}

/// Load and parse all requested MathJax JSON suites in the order the user passed them.
fn load_suites(options: &Options) -> Result<Vec<LoadedSuite>, String> {
    let mut suites = Vec::new();
    for source_path in collect_source_paths(options)? {
        eprintln!(
            "import_mathjax_json_samples: loading {}",
            source_path.display()
        );
        let content = fs::read_to_string(&source_path)
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

/// Collect explicit files plus recursively discovered directory entries into one stable source list.
fn collect_source_paths(options: &Options) -> Result<Vec<PathBuf>, String> {
    let mut paths = options.sources.clone();
    for source_dir in &options.source_dirs {
        collect_json_paths(source_dir, &mut paths)?;
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

/// Recursively collect `.json` files so package subdirectories are not skipped by accident.
fn collect_json_paths(dir: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in
        fs::read_dir(dir).map_err(|err| format!("failed to read {}: {err}", dir.display()))?
    {
        let path = entry
            .map_err(|err| format!("failed to read entry in {}: {err}", dir.display()))?
            .path();
        if path.is_dir() {
            collect_json_paths(&path, paths)?;
        } else if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            paths.push(path);
        }
    }
    Ok(())
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
) -> Result<Vec<ManifestSample>, String> {
    let all_dir = options.output_dir.join("all");
    let skipped_dir = options.output_dir.join("skipped");
    fs::create_dir_all(&all_dir)
        .map_err(|err| format!("failed to create {}: {err}", all_dir.display()))?;
    fs::create_dir_all(&skipped_dir)
        .map_err(|err| format!("failed to create {}: {err}", skipped_dir.display()))?;

    let mut manifest_samples = Vec::new();
    let mut success_count = 0usize;
    for (index, case) in selected_cases.iter().enumerate() {
        let case_number = index + 1;
        let case_id = format!("{case_number:06}");
        let wrapped_input = mathtype_tex_payload(&case.raw_input);
        let all_tex_name = format!("raw_eq_{case_id}.tex");
        let all_tex_path = all_dir.join(&all_tex_name);
        let probe_ole_path = options.output_dir.join("probe.ole.bin");
        eprintln!(
            "import_mathjax_json_samples: case={} suite={} test={}",
            case_id, case.suite_name, case.test_name
        );
        fs::write(&all_tex_path, &wrapped_input)
            .map_err(|err| format!("failed to write {}: {err}", all_tex_path.display()))?;

        match run_mathtype_helper(
            &options.helper,
            &options.pre_verb,
            &wrapped_input,
            &probe_ole_path,
            options.timeout_ms,
        ) {
            Ok(()) => {
                success_count += 1;
                let sample_id = format!("{success_count:03}");
                let tex_name = format!("eq_{sample_id}.tex");
                let ole_name = format!("mt_eq_{sample_id}.ole.bin");
                let tex_path = options.output_dir.join(&tex_name);
                let ole_path = options.output_dir.join(&ole_name);
                fs::copy(&all_tex_path, &tex_path).map_err(|err| {
                    format!(
                        "failed to copy {} to {}: {err}",
                        all_tex_path.display(),
                        tex_path.display()
                    )
                })?;
                fs::rename(&probe_ole_path, &ole_path).map_err(|err| {
                    format!(
                        "failed to move {} to {}: {err}",
                        probe_ole_path.display(),
                        ole_path.display()
                    )
                })?;
                manifest_samples.push(ManifestSample {
                    case_id,
                    sample_id: Some(sample_id),
                    status: "included".to_string(),
                    suite_name: case.suite_name.clone(),
                    test_name: case.test_name.clone(),
                    raw_input: case.raw_input.clone(),
                    wrapped_input,
                    all_tex_file: format!("all/{all_tex_name}"),
                    tex_file: Some(tex_name),
                    truth_ole_file: Some(ole_name),
                    helper_error_file: None,
                });
            }
            Err(err) if options.keep_going_on_helper_error => {
                let error_name = format!("skip_eq_{case_id}.err.txt");
                let error_path = skipped_dir.join(&error_name);
                fs::write(&error_path, &err).map_err(|write_err| {
                    format!("failed to write {}: {write_err}", error_path.display())
                })?;
                manifest_samples.push(ManifestSample {
                    case_id,
                    sample_id: None,
                    status: "helper_error".to_string(),
                    suite_name: case.suite_name.clone(),
                    test_name: case.test_name.clone(),
                    raw_input: case.raw_input.clone(),
                    wrapped_input,
                    all_tex_file: format!("all/{all_tex_name}"),
                    tex_file: None,
                    truth_ole_file: None,
                    helper_error_file: Some(format!("skipped/{error_name}")),
                });
            }
            Err(err) => return Err(err),
        }
    }
    let _ = fs::remove_file(options.output_dir.join("probe.ole.bin"));
    Ok(manifest_samples)
}

/// Write a deterministic manifest so the generated smoke set stays easy to audit.
fn write_manifest(
    loaded_suites: &[LoadedSuite],
    manifest_samples: Vec<ManifestSample>,
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
        samples: manifest_samples,
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
    use super::{collect_json_paths, collect_suite_cases, select_cases, LoadedSuite};
    use serde_json::{json, Map, Value};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

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

    /// Keep recursive source discovery aligned with the user's expectation that package subfolders count too.
    #[test]
    fn collect_json_paths_recurses_into_subdirectories() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("mathtype-rust-json-{unique}"));
        let nested = root.join("nested");
        fs::create_dir_all(&nested).expect("create nested directory");
        fs::write(root.join("top.json"), "{}").expect("write top json");
        fs::write(nested.join("inner.json"), "{}").expect("write nested json");
        fs::write(nested.join("ignore.txt"), "x").expect("write non-json");

        let mut paths = Vec::new();
        collect_json_paths(&root, &mut paths).expect("collect recursive json paths");
        paths.sort();

        assert_eq!(paths.len(), 2);
        assert!(paths.iter().any(|path| path.ends_with("top.json")));
        assert!(paths.iter().any(|path| path.ends_with("inner.json")));

        fs::remove_dir_all(&root).expect("remove temp tree");
    }
}
