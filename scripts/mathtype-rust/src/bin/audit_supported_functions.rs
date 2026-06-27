#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::env;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::thread;
use std::time::{Duration, Instant};

#[path = "../ast.rs"]
mod ast;
#[path = "../cfb.rs"]
mod cfb;
#[path = "../generated/mod.rs"]
mod generated;
#[path = "../mathtype_ansi.rs"]
mod mathtype_ansi;
#[path = "../mathtype_input.rs"]
mod mathtype_input;
#[path = "../mtef.rs"]
mod mtef;
#[path = "../parser.rs"]
mod parser;
#[path = "../raw_fallback.rs"]
mod raw_fallback;
#[path = "../typeface.rs"]
mod typeface;

use ast::Expr;
use mathtype_input::mathtype_tex_payload;
use mtef::write_mtef;
use parser::{normalize_latex, Parser};
use raw_fallback::is_known_mathtype_raw_command;

const MATHTYPE_COMPARE_CACHE_VERSION: &str = "v2";

/// Scan Supported Functions.md and classify snippets against the current parser.
fn main() -> Result<(), String> {
    let config = Config::parse(env::args().skip(1).collect())?;
    let markdown = fs::read_to_string(&config.input)
        .map_err(|err| format!("failed to read {}: {err}", config.input.display()))?;
    let snippets = filter_snippets(extract_tex_snippets(&markdown), &config);
    let math_sections =
        filter_math_snippet_sections(extract_math_snippet_sections(&markdown), &config);
    let math_snippets = math_sections.keys().cloned().collect::<Vec<_>>();
    let report = audit_snippets(&snippets);
    let math_report = audit_snippets(&math_snippets);
    if config.view.includes_code() {
        print_report(
            "supported_functions",
            &report,
            config.limit,
            config.show_static_audit,
        );
    }
    let math_raw_class = classify_raw_fallbacks(&math_report.raw_fallback);
    if config.view.includes_math() {
        print_report(
            "supported_functions_math",
            &math_report,
            config.limit,
            config.show_static_audit,
        );
        if config.show_static_audit {
            print_section_counts(
                "supported_functions_math_raw_fallback_sections",
                &math_report.raw_fallback,
                &math_sections,
                config.limit,
            );
            print_section_counts(
                "supported_functions_math_unclassified_raw_fallback_sections",
                &math_raw_class.unclassified,
                &math_sections,
                config.limit,
            );
            print_remaining_blocker_counts(
                "supported_functions_math_unclassified_raw_fallback_blockers",
                &math_raw_class.unclassified,
                config.limit,
            );
        }
    }
    if let Some(path) = &config.unclassified_jsonl {
        write_unclassified_jsonl(path, &math_raw_class.unclassified, &math_sections)?;
        println!("wrote_unclassified_jsonl={}", path.display());
    }
    if let Some(path) = &config.remaining_jsonl {
        write_remaining_jsonl(path, &math_raw_class.unclassified, &math_sections)?;
        println!("wrote_remaining_jsonl={}", path.display());
    }
    if let Some(compare) = &config.mathtype_compare {
        fs::create_dir_all(&compare.work_dir)
            .map_err(|err| format!("failed to create {}: {err}", compare.work_dir.display()))?;
        fs::create_dir_all(&compare.cache_dir)
            .map_err(|err| format!("failed to create {}: {err}", compare.cache_dir.display()))?;
        let mut probe_session = MathTypeProbeSession::default();
        if config.view.includes_code() {
            let compare_report =
                compare_snippets_with_mathtype(&snippets, compare, &mut probe_session);
            print_mathtype_compare_report(
                "supported_functions_mathtype",
                &compare_report,
                config.limit,
            );
        }
        if config.view.includes_math() {
            let compare_report =
                compare_snippets_with_mathtype(&math_snippets, compare, &mut probe_session);
            print_mathtype_compare_report(
                "supported_functions_math_mathtype",
                &compare_report,
                config.limit,
            );
        }
    }
    Ok(())
}

struct Config {
    input: PathBuf,
    limit: usize,
    max_snippets: Option<usize>,
    snippet_filter: Option<String>,
    show_static_audit: bool,
    view: AuditView,
    unclassified_jsonl: Option<PathBuf>,
    remaining_jsonl: Option<PathBuf>,
    mathtype_compare: Option<MathTypeCompareConfig>,
}

struct MathTypeCompareConfig {
    helper: PathBuf,
    work_dir: PathBuf,
    cache_dir: PathBuf,
    helper_fingerprint: String,
    pre_verb: String,
    timeout_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AuditView {
    Both,
    Code,
    Math,
}

impl AuditView {
    /// Return true when inline-code snippets should be printed.
    fn includes_code(self) -> bool {
        matches!(self, AuditView::Both | AuditView::Code)
    }

    /// Return true when complete math formulas should be printed.
    fn includes_math(self) -> bool {
        matches!(self, AuditView::Both | AuditView::Math)
    }
}

impl Config {
    /// Parse the small CLI surface used for coverage audits.
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut input = PathBuf::from(r"docs\Supported Functions.md");
        let mut limit = 80usize;
        let mut max_snippets = None;
        let mut snippet_filter = None;
        let mut show_static_audit = false;
        let mut view = AuditView::Both;
        let mut unclassified_jsonl = None;
        let mut remaining_jsonl = None;
        let mut mathtype_compare = false;
        let mut mathtype_helper = PathBuf::from(
            r"..\..\src\pandoc_manuscript\mathtype\ole_helper\bin\Release\net48\MathTypeOleHelper.exe",
        );
        let mut mathtype_work_dir = PathBuf::from(r".pmt\audit-supported-functions-mathtype");
        // Keep the default aligned with the user-required MathType helper mode.
        let mut mathtype_pre_verb = "2".to_string();
        let mut mathtype_timeout_ms = 30_000u64;
        let mut index = 0;
        while index < args.len() {
            match args[index].as_str() {
                "--input" => {
                    index += 1;
                    input = PathBuf::from(
                        args.get(index)
                            .ok_or_else(|| "--input requires a path".to_string())?,
                    );
                }
                "--limit" => {
                    index += 1;
                    let raw = args
                        .get(index)
                        .ok_or_else(|| "--limit requires a number".to_string())?;
                    limit = raw
                        .parse::<usize>()
                        .map_err(|err| format!("invalid --limit {raw}: {err}"))?;
                }
                "--max-snippets" => {
                    index += 1;
                    let raw = args
                        .get(index)
                        .ok_or_else(|| "--max-snippets requires a number".to_string())?;
                    max_snippets = Some(
                        raw.parse::<usize>()
                            .map_err(|err| format!("invalid --max-snippets {raw}: {err}"))?,
                    );
                }
                "--snippet-filter" => {
                    index += 1;
                    snippet_filter = Some(
                        args.get(index)
                            .ok_or_else(|| "--snippet-filter requires text".to_string())?
                            .clone(),
                    );
                }
                "--view" => {
                    index += 1;
                    let raw = args
                        .get(index)
                        .ok_or_else(|| "--view requires both, code, or math".to_string())?;
                    view = parse_audit_view(raw)?;
                }
                "--show-static-audit" => show_static_audit = true,
                "--math-only" => view = AuditView::Math,
                "--code-only" => view = AuditView::Code,
                "--mathtype-compare" => mathtype_compare = true,
                "--helper" => {
                    index += 1;
                    mathtype_helper = PathBuf::from(
                        args.get(index)
                            .ok_or_else(|| "--helper requires a path".to_string())?,
                    );
                }
                "--work-dir" => {
                    index += 1;
                    mathtype_work_dir = PathBuf::from(
                        args.get(index)
                            .ok_or_else(|| "--work-dir requires a path".to_string())?,
                    );
                }
                "--pre-verb" => {
                    index += 1;
                    mathtype_pre_verb =
                        require_pre_verb_two_arg(args.get(index).ok_or_else(|| {
                            "--pre-verb requires an OLE verb number".to_string()
                        })?)?;
                }
                "--timeout-ms" => {
                    index += 1;
                    let raw = args
                        .get(index)
                        .ok_or_else(|| "--timeout-ms requires a number".to_string())?;
                    mathtype_timeout_ms = raw
                        .parse::<u64>()
                        .map_err(|err| format!("invalid --timeout-ms {raw}: {err}"))?;
                }
                "--write-unclassified-jsonl" => {
                    index += 1;
                    unclassified_jsonl = Some(PathBuf::from(args.get(index).ok_or_else(|| {
                        "--write-unclassified-jsonl requires a path".to_string()
                    })?));
                }
                "--write-remaining-jsonl" => {
                    index += 1;
                    remaining_jsonl =
                        Some(PathBuf::from(args.get(index).ok_or_else(|| {
                            "--write-remaining-jsonl requires a path".to_string()
                        })?));
                }
                other => return Err(format!("unknown option: {other}\n{}", usage())),
            }
            index += 1;
        }
        Ok(Self {
            input,
            limit,
            max_snippets,
            snippet_filter,
            show_static_audit,
            view,
            unclassified_jsonl,
            remaining_jsonl,
            mathtype_compare: if mathtype_compare {
                Some(MathTypeCompareConfig {
                    cache_dir: mathtype_work_dir.join("cache"),
                    helper_fingerprint: file_fingerprint(&mathtype_helper)?,
                    helper: mathtype_helper,
                    work_dir: mathtype_work_dir,
                    pre_verb: mathtype_pre_verb,
                    timeout_ms: mathtype_timeout_ms,
                })
            } else {
                None
            },
        })
    }
}

/// Return the usage text for invalid audit invocations.
fn usage() -> &'static str {
    "Usage: audit_supported_functions [--input <Supported Functions.md>] [--limit <N>] [--max-snippets <N>] [--snippet-filter <text>] [--view both|code|math] [--show-static-audit] [--math-only] [--code-only] [--write-unclassified-jsonl <path>] [--write-remaining-jsonl <path>] [--mathtype-compare] [--helper <exe>] [--work-dir <dir>] [--pre-verb 2] [--timeout-ms <N>]"
}

/// Parse a report-view selector from the CLI.
fn parse_audit_view(raw: &str) -> Result<AuditView, String> {
    match raw {
        "both" => Ok(AuditView::Both),
        "code" => Ok(AuditView::Code),
        "math" => Ok(AuditView::Math),
        _ => Err(format!(
            "invalid --view {raw}; expected both, code, or math"
        )),
    }
}

#[derive(Default)]
struct AuditReport {
    total: usize,
    native_parse: Vec<String>,
    native_render: Vec<String>,
    raw_fallback: Vec<String>,
    raw_render: Vec<String>,
    raw_render_error: Vec<(String, String)>,
    render_error: Vec<(String, String)>,
    syntax_fragment: Vec<(String, Option<String>)>,
    parse_error: Vec<(String, String)>,
}

struct RawFallbackClass {
    known: Vec<String>,
    unclassified: Vec<String>,
}

#[derive(Default)]
struct MathTypeCompareReport {
    total: usize,
    cache_stats: MathTypeProbeStats,
    matched: Vec<String>,
    mismatched: Vec<MathTypeMismatch>,
    helper_error: Vec<(String, String)>,
    rust_error: Vec<(String, String)>,
    skipped: Vec<(String, String)>,
}

struct MathTypeMismatch {
    snippet: String,
    mathtype_len: usize,
    rust_len: usize,
    first_diff: usize,
}

struct MathTypeCachePaths {
    mtef: PathBuf,
    error: PathBuf,
}

#[derive(Clone, Copy, Default)]
struct MathTypeProbeStats {
    memory_hits: usize,
    disk_hits: usize,
    misses: usize,
    helper_invocations: usize,
}

impl MathTypeProbeStats {
    /// Return the per-report delta from one shared probe session snapshot.
    fn delta_since(self, earlier: Self) -> Self {
        Self {
            memory_hits: self.memory_hits.saturating_sub(earlier.memory_hits),
            disk_hits: self.disk_hits.saturating_sub(earlier.disk_hits),
            misses: self.misses.saturating_sub(earlier.misses),
            helper_invocations: self
                .helper_invocations
                .saturating_sub(earlier.helper_invocations),
        }
    }
}

#[derive(Default)]
struct MathTypeProbeSession {
    memo: HashMap<String, CachedMathTypeProbe>,
    stats: MathTypeProbeStats,
}

impl MathTypeProbeSession {
    /// Snapshot cumulative stats so code/math views can print their own deltas.
    fn stats_snapshot(&self) -> MathTypeProbeStats {
        self.stats
    }
}

#[derive(Clone)]
enum CachedMathTypeProbe {
    Success(Vec<u8>),
    Error(String),
}

/// Extract unique inline-code snippets that contain TeX control sequences.
fn extract_tex_snippets(markdown: &str) -> Vec<String> {
    let mut snippets = BTreeSet::new();
    let mut chars = markdown.chars().peekable();
    let mut in_code = false;
    let mut current = String::new();
    while let Some(ch) = chars.next() {
        if ch == '\u{60}' {
            if chars.peek() == Some(&'\u{60}') {
                continue;
            }
            if in_code {
                add_tex_snippet(&current, &mut snippets);
                current.clear();
            }
            in_code = !in_code;
        } else if in_code {
            current.push(ch);
        }
    }
    snippets.into_iter().collect()
}

/// Extract unique Markdown math spans as complete formulas for coverage audits.
fn extract_math_snippets(markdown: &str) -> Vec<String> {
    extract_math_snippet_sections(markdown)
        .into_keys()
        .collect::<Vec<_>>()
}

/// Extract Markdown math spans and the Supported Functions section they came from.
fn extract_math_snippet_sections(markdown: &str) -> BTreeMap<String, String> {
    let mut snippets = BTreeMap::new();
    let chars = markdown.chars().collect::<Vec<_>>();
    let sections = section_by_char(markdown);
    let mut index = 0usize;
    while index < chars.len() {
        if chars[index] == '\u{60}' {
            index = skip_code_span(&chars, index);
            continue;
        }
        if chars[index] != '$' {
            index += 1;
            continue;
        }
        let delimiter_len = if chars.get(index + 1) == Some(&'$') {
            2
        } else {
            1
        };
        let start = index + delimiter_len;
        if let Some(end) = find_math_span_end(&chars, start, delimiter_len) {
            if let Some(snippet) = clean_math_snippet(&chars[start..end].iter().collect::<String>())
            {
                let section = sections
                    .get(start)
                    .cloned()
                    .unwrap_or_else(|| "<unknown>".to_string());
                snippets.entry(snippet).or_insert(section);
            }
            index = end + delimiter_len;
        } else {
            index += delimiter_len;
        }
    }
    snippets
}

/// Apply optional snippet filtering so compare/debug runs can stay focused.
fn filter_snippets(mut snippets: Vec<String>, config: &Config) -> Vec<String> {
    if let Some(filter) = &config.snippet_filter {
        snippets.retain(|snippet| snippet.contains(filter));
    }
    if let Some(max) = config.max_snippets {
        snippets.truncate(max);
    }
    snippets
}

/// Apply the same focus filter to math snippets while preserving section labels.
fn filter_math_snippet_sections(
    snippets: BTreeMap<String, String>,
    config: &Config,
) -> BTreeMap<String, String> {
    let filtered = snippets.into_iter().filter(|(snippet, _)| {
        config
            .snippet_filter
            .as_ref()
            .is_none_or(|filter| snippet.contains(filter))
    });
    match config.max_snippets {
        Some(max) => filtered.take(max).collect(),
        None => filtered.collect(),
    }
}

/// Return the active Markdown heading at each character offset.
fn section_by_char(markdown: &str) -> Vec<String> {
    let mut sections = Vec::new();
    let mut current_section = "<preamble>".to_string();
    for line in markdown.split_inclusive('\n') {
        let heading_line = line.trim_end_matches('\n').trim_end_matches('\r');
        if let Some(heading) = markdown_heading(heading_line) {
            current_section = heading.to_string();
        }
        sections.extend(line.chars().map(|_| current_section.clone()));
    }
    sections
}

/// Skip a Markdown code span, including doubled backtick delimiters.
fn skip_code_span(chars: &[char], start: usize) -> usize {
    let delimiter_len = count_repeated(chars, start, '\u{60}');
    let mut index = start + delimiter_len;
    while index < chars.len() {
        if chars[index] == '\u{60}' && count_repeated(chars, index, '\u{60}') >= delimiter_len {
            return index + delimiter_len;
        }
        index += 1;
    }
    start + delimiter_len
}

/// Add one code span when it looks like a TeX expression worth auditing.
fn add_tex_snippet(raw: &str, snippets: &mut BTreeSet<String>) {
    let snippet = raw.trim();
    if snippet.contains('\\') && !snippet.contains("https://") && !snippet.contains('…') {
        snippets.insert(snippet.to_string());
    }
}

/// Find a matching `$` or `$$` delimiter that is not escaped.
fn find_math_span_end(chars: &[char], start: usize, delimiter_len: usize) -> Option<usize> {
    let mut index = start;
    let mut brace_depth = 0usize;
    while index < chars.len() {
        if delimiter_len == 1 && matches!(chars[index], '\n' | '\r') {
            return None;
        }
        if chars[index] == '\\' {
            index += 2;
            continue;
        }
        match chars[index] {
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            _ => {}
        }
        if brace_depth == 0
            && delimiter_len == 2
            && chars[index] == '$'
            && chars.get(index + 1) == Some(&'$')
        {
            return Some(index);
        }
        if brace_depth == 0 && delimiter_len == 1 && chars[index] == '$' {
            return Some(index);
        }
        index += 1;
    }
    None
}

/// Count repeated delimiter characters from an index.
fn count_repeated(chars: &[char], start: usize, needle: char) -> usize {
    let mut index = start;
    while chars.get(index) == Some(&needle) {
        index += 1;
    }
    index - start
}

/// Add one complete math span when it is useful as parser/writer coverage.
fn clean_math_snippet(raw: &str) -> Option<String> {
    let snippet = raw.trim();
    if !snippet.is_empty()
        && !snippet.contains("https://")
        && !snippet.contains('…')
        && !snippet.contains("<span")
        && !snippet.contains('\u{60}')
        && !snippet.contains("\n\n")
    {
        Some(snippet.to_string())
    } else {
        None
    }
}

/// Return a Markdown heading's display text.
fn markdown_heading(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let level_end = trimmed.chars().take_while(|ch| *ch == '#').count();
    if level_end == 0 || trimmed.as_bytes().get(level_end) != Some(&b' ') {
        return None;
    }
    Some(trimmed[level_end..].trim())
}

/// Classify snippets according to the current parser's AST.
fn audit_snippets(snippets: &[String]) -> AuditReport {
    let mut report = AuditReport {
        total: snippets.len(),
        ..AuditReport::default()
    };
    for snippet in snippets {
        if is_syntax_fragment(snippet) {
            report.syntax_fragment.push((snippet.clone(), None));
            continue;
        }
        let normalized = normalize_latex(snippet);
        match Parser::new(&normalized).parse() {
            Ok(expr) if expr.contains_raw_tex() => {
                report.raw_fallback.push(snippet.clone());
                match write_mtef(&normalized, &expr) {
                    Ok(_) => report.raw_render.push(snippet.clone()),
                    Err(err) => report.raw_render_error.push((snippet.clone(), err)),
                }
            }
            Ok(expr) => {
                report.native_parse.push(snippet.clone());
                match write_mtef(&normalized, &expr) {
                    Ok(_) => report.native_render.push(snippet.clone()),
                    Err(err) => report.render_error.push((snippet.clone(), err)),
                }
            }
            Err(err) if is_documentation_fragment(snippet, &err) => {
                report.syntax_fragment.push((snippet.clone(), Some(err)));
            }
            Err(err) => report.parse_error.push((snippet.clone(), err)),
        }
    }
    report
}

/// Compare renderable snippets against live MathType output through the COM helper.
fn compare_snippets_with_mathtype(
    snippets: &[String],
    config: &MathTypeCompareConfig,
    probe_session: &mut MathTypeProbeSession,
) -> MathTypeCompareReport {
    let stats_before = probe_session.stats_snapshot();
    let mut report = MathTypeCompareReport {
        total: snippets.len(),
        ..MathTypeCompareReport::default()
    };
    for (index, snippet) in snippets.iter().enumerate() {
        if is_syntax_fragment(snippet) {
            report
                .skipped
                .push((snippet.clone(), "syntax fragment".to_string()));
            continue;
        }
        let normalized = normalize_latex(snippet);
        // Compare Rust and MathType against the same original LaTeX so helper
        // fallbacks such as `aligned` stay visible in the byte-level audit.
        let compare_latex = normalized;
        let expr = match Parser::new(&compare_latex).parse() {
            Ok(expr) => expr,
            Err(err) if is_documentation_fragment(snippet, &err) => {
                report.skipped.push((snippet.clone(), err));
                continue;
            }
            Err(err) => {
                report.rust_error.push((snippet.clone(), err));
                continue;
            }
        };
        let rust_mtef = match write_mtef(&compare_latex, &expr) {
            Ok(bytes) => bytes,
            Err(err) => {
                report.rust_error.push((snippet.clone(), err));
                continue;
            }
        };
        let mathtype_mtef = match probe_mathtype_mtef(&compare_latex, config, index, probe_session)
        {
            Ok(bytes) => bytes,
            Err(err) => {
                report.helper_error.push((snippet.clone(), err));
                continue;
            }
        };
        if rust_mtef == mathtype_mtef {
            report.matched.push(snippet.clone());
        } else {
            report.mismatched.push(MathTypeMismatch {
                snippet: snippet.clone(),
                mathtype_len: mathtype_mtef.len(),
                rust_len: rust_mtef.len(),
                first_diff: first_diff(&mathtype_mtef, &rust_mtef),
            });
        }
    }
    report.cache_stats = probe_session.stats_snapshot().delta_since(stats_before);
    report
}

/// Encode one formula with MathType and return the native MTEF payload.
fn probe_mathtype_mtef(
    latex: &str,
    config: &MathTypeCompareConfig,
    index: usize,
    probe_session: &mut MathTypeProbeSession,
) -> Result<Vec<u8>, String> {
    let payload = mathtype_tex_payload(latex);
    let key = cache_key(&payload, config);
    if let Some(cached) = probe_session.memo.get(&key).cloned() {
        probe_session.stats.memory_hits += 1;
        return match cached {
            CachedMathTypeProbe::Success(bytes) => Ok(bytes),
            CachedMathTypeProbe::Error(err) => Err(err),
        };
    }
    let cache_paths = mathtype_cache_paths(&key, config);
    if let Some(cached) = read_cached_mathtype_probe(&cache_paths)? {
        probe_session.stats.disk_hits += 1;
        probe_session.memo.insert(key, cached.clone());
        return match cached {
            CachedMathTypeProbe::Success(bytes) => Ok(bytes),
            CachedMathTypeProbe::Error(err) => Err(err),
        };
    }
    probe_session.stats.misses += 1;
    let probe_dir = config
        .work_dir
        .join(format!("run-{}-{index:04}", process::id()));
    fs::create_dir_all(&probe_dir)
        .map_err(|err| format!("failed to create {}: {err}", probe_dir.display()))?;
    let tex_path = probe_dir.join("probe.tex");
    let ole_path = probe_dir.join("probe.ole.bin");
    fs::write(&tex_path, &payload)
        .map_err(|err| format!("failed to write {}: {err}", tex_path.display()))?;
    let result = (|| {
        probe_session.stats.helper_invocations += 1;
        run_mathtype_helper(
            &config.helper,
            &config.pre_verb,
            &payload,
            &ole_path,
            config.timeout_ms,
        )?;
        let ole = fs::read(&ole_path)
            .map_err(|err| format!("failed to read {}: {err}", ole_path.display()))?;
        extract_mtef_from_ole(&ole)
    })();
    match result {
        Ok(mtef) => {
            write_cached_mathtype_mtef(&cache_paths, &mtef)?;
            probe_session
                .memo
                .insert(key, CachedMathTypeProbe::Success(mtef.clone()));
            Ok(mtef)
        }
        Err(err) => {
            write_cached_mathtype_error(&cache_paths, &err)?;
            probe_session
                .memo
                .insert(key, CachedMathTypeProbe::Error(err.clone()));
            Err(err)
        }
    }
}

/// Build stable cache paths from the exact helper payload and helper fingerprint.
fn mathtype_cache_paths(key: &str, config: &MathTypeCompareConfig) -> MathTypeCachePaths {
    let root = config.cache_dir.join(&key[..2]).join(key);
    MathTypeCachePaths {
        mtef: root.join("mtef.bin"),
        error: root.join("error.txt"),
    }
}

/// Build a stable cache key for one MathType helper invocation.
fn cache_key(payload: &str, config: &MathTypeCompareConfig) -> String {
    // Bump this when the caller-side helper contract changes. `v2` starts the
    // cache lineage that passes literal TeX payload text to `--input` instead
    // of a temp file path, so older probe results cannot masquerade as current
    // MathType output in byte-for-byte audits.
    let material = format!(
        "{MATHTYPE_COMPARE_CACHE_VERSION}\0payload={payload}\0pre_verb={}\0helper={}",
        config.pre_verb.as_str(),
        config.helper_fingerprint
    );
    fnv1a64_hex(material.as_bytes())
}

/// Return true for helper failures that should be retried instead of cached.
fn is_transient_mathtype_probe_error(err: &str) -> bool {
    let err = err.to_ascii_lowercase();
    err.contains("timed out")
        || err.contains("failed to run")
        || err.contains("failed while waiting for helper")
        // Helper exit code 1 is too coarse to cache safely: transient COM
        // crashes and real formula failures both collapse to the same status.
        || err.contains("status exit code: 1")
        || err.contains("0x800706be")
        || err.contains("0x80010105")
        || err.contains("rpc_e_serverfault")
        || err.contains("远程过程调用失败")
        || err.contains("服务器出现意外情况")
}

/// Load a cached MathType probe result so repeated audits do not relaunch the helper.
fn read_cached_mathtype_probe(
    paths: &MathTypeCachePaths,
) -> Result<Option<CachedMathTypeProbe>, String> {
    match fs::read(&paths.mtef) {
        Ok(bytes) if !bytes.is_empty() => return Ok(Some(CachedMathTypeProbe::Success(bytes))),
        Ok(_) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            return Err(format!(
                "failed to read cache {}: {err}",
                paths.mtef.display()
            ));
        }
    }
    match fs::read_to_string(&paths.error) {
        Ok(message) => {
            let message = if message.trim().is_empty() {
                "cached MathType probe failed without details".to_string()
            } else {
                message
            };
            if is_transient_mathtype_probe_error(&message) {
                let _ = fs::remove_file(&paths.error);
                return Ok(None);
            }
            Ok(Some(CachedMathTypeProbe::Error(message)))
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(format!(
            "failed to read cache {}: {err}",
            paths.error.display()
        )),
    }
}

/// Store one successful MathType MTEF payload for later audit runs.
fn write_cached_mathtype_mtef(paths: &MathTypeCachePaths, mtef: &[u8]) -> Result<(), String> {
    if let Some(parent) = paths.mtef.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    let _ = fs::remove_file(&paths.error);
    fs::write(&paths.mtef, mtef)
        .map_err(|err| format!("failed to write cache {}: {err}", paths.mtef.display()))
}

/// Store one helper failure so unsupported snippets do not keep reopening MathType.
fn write_cached_mathtype_error(paths: &MathTypeCachePaths, err: &str) -> Result<(), String> {
    let _ = fs::remove_file(&paths.mtef);
    if is_transient_mathtype_probe_error(err) {
        let _ = fs::remove_file(&paths.error);
        return Ok(());
    }
    if let Some(parent) = paths.error.parent() {
        fs::create_dir_all(parent)
            .map_err(|cache_err| format!("failed to create {}: {cache_err}", parent.display()))?;
    }
    fs::write(&paths.error, err).map_err(|cache_err| {
        format!(
            "failed to write cache {}: {cache_err}",
            paths.error.display()
        )
    })
}

/// Hash one file into a stable helper fingerprint for cache invalidation.
fn file_fingerprint(path: &Path) -> Result<String, String> {
    let bytes =
        fs::read(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    Ok(fnv1a64_hex(&bytes))
}

/// Return a deterministic hex digest without pulling in an external hashing crate.
fn fnv1a64_hex(bytes: &[u8]) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{hash:016x}")
}

/// Extract MathType's Equation Native payload after the fixed OLE stream header.
fn extract_mtef_from_ole(ole: &[u8]) -> Result<Vec<u8>, String> {
    let equation_native = cfb::read_regular_stream(ole, "Equation Native")?;
    Ok(equation_native
        .get(28..)
        .ok_or_else(|| "Equation Native stream is shorter than the native header".to_string())?
        .to_vec())
}

/// Invoke the existing MathType COM helper for one TeX input file.
fn run_mathtype_helper(
    helper: &PathBuf,
    pre_verb: &str,
    tex_payload: &str,
    ole_path: &PathBuf,
    timeout_ms: u64,
) -> Result<(), String> {
    let _ = fs::remove_file(ole_path);
    let stderr_path = ole_path.with_extension("stderr.txt");
    let _ = fs::remove_file(&stderr_path);
    let mut command = Command::new(helper);
    command.args(["--method", "set-data"]);
    command.args(["--pre-verb", pre_verb]);
    // Persist helper stderr so transient COM/OLE failures can be classified and
    // excluded from the long-lived on-disk probe cache.
    command.stderr(
        File::create(&stderr_path)
            .map_err(|err| format!("failed to create {}: {err}", stderr_path.display()))?,
    );
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
        let helper_stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
        let _ = fs::remove_file(&stderr_path);
        let helper_stderr = helper_stderr.trim();
        return Err(format!(
            "{} failed for {} with status {status}{}",
            helper.display(),
            tex_payload,
            if helper_stderr.is_empty() {
                String::new()
            } else {
                format!("; stderr={helper_stderr}")
            }
        ));
    }
    let _ = fs::remove_file(&stderr_path);
    Ok(())
}

/// Reject unsupported helper verbs so audit cache keys stay tied to the validated MathType path.
fn require_pre_verb_two_arg(pre_verb: &str) -> Result<String, String> {
    if pre_verb == "2" {
        Ok("2".to_string())
    } else {
        Err("--pre-verb only supports value 2.".to_string())
    }
}

/// Wait for the helper so one unsupported MathType input cannot hang the audit.
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

/// Return the first byte offset that differs between two MTEF payloads.
fn first_diff(left: &[u8], right: &[u8]) -> usize {
    left.iter()
        .zip(right.iter())
        .position(|(left, right)| left != right)
        .unwrap_or_else(|| left.len().min(right.len()))
}

/// Return true for documentation fragments that are not standalone formulas.
fn is_syntax_fragment(snippet: &str) -> bool {
    let trimmed = snippet.trim();
    is_placeholder_code_span(trimmed)
        || is_bare_incomplete_command(trimmed)
        || is_text_mode_accent_fragment(trimmed)
        || is_non_rendering_spacing_fragment(trimmed)
        || trimmed.starts_with("\\@")
        || trimmed.starts_with('@')
        || trimmed.starts_with("\\begin{") && !trimmed.contains("\\end{")
        || trimmed.starts_with("\\end{")
        || matches!(
            trimmed,
            "\\left" | "\\left." | "\\right" | "\\right." | "\\color"
        )
        || trimmed.ends_with('\\')
        || trimmed.ends_with('(')
        || (trimmed.starts_with('$') ^ trimmed.ends_with('$'))
        || (trimmed.contains('&') && !trimmed.contains("\\begin{"))
        || trimmed.ends_with("_{")
        || trimmed.contains("\\begin{") && !trimmed.contains("\\end{")
}

/// Return true for bare text-accent examples that are documented only inside \text{...}.
fn is_text_mode_accent_fragment(trimmed: &str) -> bool {
    [
        "\\'{a}", "\\\"{a}", "\\.{a}", "\\={a}", "\\`{a}", "\\^{a}", "\\~{a}",
    ]
    .contains(&trimmed)
}

/// Return true for standalone spacing examples that do not form an inspectable visible formula.
fn is_non_rendering_spacing_fragment(trimmed: &str) -> bool {
    [
        "\\!",
        "\\,",
        "\\:",
        "\\;",
        "\\>",
        "\\<space>",
        "\\space",
        "\\thinspace",
        "\\medspace",
        "\\thickspace",
        "\\negthinspace",
        "\\negmedspace",
        "\\negthickspace",
        "\\nobreakspace",
    ]
    .contains(&trimmed)
}

/// Return true for code spans that document syntax placeholders, not formulas.
fn is_placeholder_code_span(trimmed: &str) -> bool {
    [
        "content",
        "definition",
        "distance",
        "macroname",
        "numargs",
        "textXX",
        "TextOrMath",
    ]
    .iter()
    .any(|placeholder| trimmed.contains(placeholder))
}

/// Return true for command names that need surrounding TeX syntax to be meaningful.
fn is_bare_incomplete_command(trimmed: &str) -> bool {
    if trimmed.starts_with("\\global\\")
        || trimmed.starts_with("\\futurelet")
        || trimmed.starts_with("\\let")
    {
        return true;
    }
    matches!(
        trimmed,
        "\\char"
            | "\\clap"
            | "\\cline"
            | "\\colorbox"
            | "\\cr"
            | "\\expandafter"
            | "\\fcolorbox"
            | "\\gdef"
            | "\\hspace"
            | "\\html"
            | "\\includegraphics"
            | "\\limits"
            | "\\llap"
            | "\\long"
            | "\\makeatletter"
            | "\\mathbin"
            | "\\mathchoice"
            | "\\mathclose"
            | "\\mathinner"
            | "\\mathop"
            | "\\mathopen"
            | "\\mathord"
            | "\\mathpunct"
            | "\\middle"
            | "\\mkern"
            | "\\mskip"
            | "\\multicolumn"
            | "\\newline"
            | "\\nobreak"
            | "\\noexpand"
            | "\\par"
            | "\\pmb"
            | "\\raisebox"
            | "\\relax"
            | "\\rlap"
            | "\\vcenter"
            | "\\xdef"
    )
}

/// Return true for documentation fragments that are not complete formulas.
fn is_documentation_fragment(snippet: &str, err: &str) -> bool {
    let trimmed = snippet.trim();
    trimmed.starts_with("\\begin{")
        || trimmed.starts_with("\\end{")
        || trimmed.starts_with("\\@")
        || trimmed.starts_with('@')
        || matches!(
            trimmed,
            "\\left" | "\\left." | "\\right" | "\\right." | "\\color"
        )
        || trimmed.ends_with('\\')
        || trimmed.ends_with('(')
        || (trimmed.starts_with('$') ^ trimmed.ends_with('$'))
        || (trimmed.contains('&') && !trimmed.contains("\\begin{"))
        || trimmed.ends_with('{')
        || err.contains("found None")
}

/// Print static-audit totals, keeping raw-fallback diagnostics behind an explicit flag.
fn print_report(prefix: &str, report: &AuditReport, limit: usize, show_static_audit: bool) {
    println!("{prefix}_snippets={}", report.total);
    println!("{prefix}_native_parse={}", report.native_parse.len());
    println!("{prefix}_native_render={}", report.native_render.len());
    println!("{prefix}_raw_render={}", report.raw_render.len());
    println!(
        "{prefix}_raw_render_error={}",
        report.raw_render_error.len()
    );
    println!("{prefix}_render_error={}", report.render_error.len());
    println!("{prefix}_syntax_fragment={}", report.syntax_fragment.len());
    println!("{prefix}_parse_error={}", report.parse_error.len());
    if show_static_audit {
        // Keep raw-fallback coverage details opt-in so MathType-compare runs
        // stay focused on byte-level matched/mismatched results.
        let raw_class = classify_raw_fallbacks(&report.raw_fallback);
        println!("{prefix}_raw_fallback={}", report.raw_fallback.len());
        println!(
            "{prefix}_known_mathtype_raw_fallback={}",
            raw_class.known.len()
        );
        println!(
            "{prefix}_unclassified_raw_fallback={}",
            raw_class.unclassified.len()
        );
        print_list(
            &format!("{prefix}_raw_fallback_examples"),
            report.raw_fallback.iter().map(|item| (item, None)),
            limit,
        );
        print_raw_command_groups(
            &format!("{prefix}_raw_fallback_command_groups"),
            &report.raw_fallback,
            limit,
        );
        print_list(
            &format!("{prefix}_unclassified_raw_fallback_examples"),
            raw_class.unclassified.iter().map(|item| (item, None)),
            limit,
        );
        print_raw_command_groups(
            &format!("{prefix}_unclassified_raw_fallback_command_groups"),
            &raw_class.unclassified,
            limit,
        );
    }
    print_list(
        &format!("{prefix}_raw_render_error_examples"),
        report
            .raw_render_error
            .iter()
            .map(|(snippet, err)| (snippet, Some(err))),
        limit,
    );
    print_list(
        &format!("{prefix}_render_error_examples"),
        report
            .render_error
            .iter()
            .map(|(snippet, err)| (snippet, Some(err))),
        limit,
    );
    print_list(
        &format!("{prefix}_syntax_fragment_examples"),
        report
            .syntax_fragment
            .iter()
            .map(|(snippet, err)| (snippet, err.as_ref())),
        limit,
    );
    print_list(
        &format!("{prefix}_parse_error_examples"),
        report
            .parse_error
            .iter()
            .map(|(snippet, err)| (snippet, Some(err))),
        limit,
    );
}

/// Print live-MathType comparison totals and representative failures.
fn print_mathtype_compare_report(prefix: &str, report: &MathTypeCompareReport, limit: usize) {
    println!("{prefix}_snippets={}", report.total);
    println!("{prefix}_matched={}", report.matched.len());
    println!("{prefix}_mismatched={}", report.mismatched.len());
    println!("{prefix}_helper_error={}", report.helper_error.len());
    println!("{prefix}_rust_error={}", report.rust_error.len());
    println!("{prefix}_skipped={}", report.skipped.len());
    println!(
        "{prefix}_cache_memory_hits={}",
        report.cache_stats.memory_hits
    );
    println!("{prefix}_cache_disk_hits={}", report.cache_stats.disk_hits);
    println!("{prefix}_cache_misses={}", report.cache_stats.misses);
    println!(
        "{prefix}_helper_invocations={}",
        report.cache_stats.helper_invocations
    );
    println!("{prefix}_mismatch_examples:");
    for (index, mismatch) in report.mismatched.iter().enumerate() {
        if index >= limit {
            println!("  ... {} more", report.mismatched.len() - limit);
            break;
        }
        println!(
            "  - {} => mathtype_len={}, rust_len={}, first_diff={}",
            mismatch.snippet, mismatch.mathtype_len, mismatch.rust_len, mismatch.first_diff
        );
    }
    print_list(
        &format!("{prefix}_helper_error_examples"),
        report
            .helper_error
            .iter()
            .map(|(snippet, err)| (snippet, Some(err))),
        limit,
    );
    print_list(
        &format!("{prefix}_rust_error_examples"),
        report
            .rust_error
            .iter()
            .map(|(snippet, err)| (snippet, Some(err))),
        limit,
    );
    print_list(
        &format!("{prefix}_skipped_examples"),
        report
            .skipped
            .iter()
            .map(|(snippet, err)| (snippet, Some(err))),
        limit,
    );
}

/// Split raw fallback into documented MathType behavior and still-unclassified gaps.
fn classify_raw_fallbacks(snippets: &[String]) -> RawFallbackClass {
    let mut known = Vec::new();
    let mut unclassified = Vec::new();
    for snippet in snippets {
        let raw_commands = raw_fallback_commands(snippet);
        if !raw_commands.is_empty()
            && raw_commands
                .iter()
                .all(|command| is_known_mathtype_raw_command(command))
        {
            known.push(snippet.clone());
        } else {
            unclassified.push(snippet.clone());
        }
    }
    RawFallbackClass {
        known,
        unclassified,
    }
}

/// Return raw TeX command names produced by the parser for one snippet.
fn raw_fallback_commands(snippet: &str) -> Vec<String> {
    let normalized = normalize_latex(snippet);
    let Ok(expr) = Parser::new(&normalized).parse() else {
        return Vec::new();
    };
    let mut commands = Vec::new();
    collect_raw_commands(&expr, &mut commands);
    commands.sort();
    commands.dedup();
    commands
}

/// Collect command names from Expr::RawTex nodes, preserving raw fallback evidence.
fn collect_raw_commands(expr: &Expr, commands: &mut Vec<String>) {
    match expr {
        Expr::RawTex(text) => {
            if let Some(command) = raw_text_command(text) {
                commands.push(command);
            }
        }
        Expr::Sequence(items) => items
            .iter()
            .for_each(|item| collect_raw_commands(item, commands)),
        Expr::Color { content, .. }
        | Expr::Style { content, .. }
        | Expr::Font { content, .. }
        | Expr::Accent { content, .. }
        | Expr::ArrowAccent { content, .. }
        | Expr::BarTemplate { content, .. }
        | Expr::Strike { content, .. }
        | Expr::NotRelation(content)
        | Expr::Sqrt(content)
        | Expr::Delimited { content, .. } => collect_raw_commands(content, commands),
        Expr::Script { base, sub, sup } => {
            collect_raw_commands(base, commands);
            if let Some(sub) = sub {
                collect_raw_commands(sub, commands);
            }
            if let Some(sup) = sup {
                collect_raw_commands(sup, commands);
            }
        }
        Expr::XArrow { label, under, .. } => {
            collect_raw_commands(label, commands);
            if let Some(under) = under {
                collect_raw_commands(under, commands);
            }
        }
        Expr::Fraction(left, right)
        | Expr::Stackrel {
            upper: left,
            lower: right,
        }
        | Expr::Underset {
            lower: left,
            base: right,
        }
        | Expr::Pile {
            upper: left,
            lower: right,
            ..
        } => {
            collect_raw_commands(left, commands);
            collect_raw_commands(right, commands);
        }
        Expr::NthRoot { index, radicand } => {
            collect_raw_commands(index, commands);
            collect_raw_commands(radicand, commands);
        }
        Expr::BigOp {
            lower, upper, body, ..
        }
        | Expr::IntegralOp {
            lower, upper, body, ..
        } => {
            lower
                .as_deref()
                .into_iter()
                .for_each(|expr| collect_raw_commands(expr, commands));
            upper
                .as_deref()
                .into_iter()
                .for_each(|expr| collect_raw_commands(expr, commands));
            body.as_deref()
                .into_iter()
                .for_each(|expr| collect_raw_commands(expr, commands));
        }
        Expr::Limit { lower, upper, .. } => {
            lower
                .as_deref()
                .into_iter()
                .for_each(|expr| collect_raw_commands(expr, commands));
            upper
                .as_deref()
                .into_iter()
                .for_each(|expr| collect_raw_commands(expr, commands));
        }
        Expr::Brace {
            content,
            annotation,
            ..
        } => {
            collect_raw_commands(content, commands);
            annotation
                .as_deref()
                .into_iter()
                .for_each(|expr| collect_raw_commands(expr, commands));
        }
        Expr::Substack { rows }
        | Expr::Subarray { rows, .. }
        | Expr::Matrix { rows, .. }
        | Expr::Environment { rows, .. } => rows
            .iter()
            .flat_map(|row| row.iter())
            .for_each(|expr| collect_raw_commands(expr, commands)),
        Expr::Char(_)
        | Expr::MarkedChar(_)
        | Expr::CommandSymbol { .. }
        | Expr::BigSymbol(_)
        | Expr::SumOperatorSymbol(_)
        | Expr::Space(_)
        | Expr::FunctionName(_)
        | Expr::Text(_)
        | Expr::Integral { .. } => {}
    }
}

/// Extract the command name from the raw TeX text emitted by parser fallback.
fn raw_text_command(text: &str) -> Option<String> {
    let chars = text.chars().collect::<Vec<_>>();
    if chars.first() != Some(&'\\') {
        return None;
    }
    let mut end = 1usize;
    while chars.get(end).is_some_and(|ch| ch.is_ascii_alphabetic()) {
        end += 1;
    }
    (end > 1).then(|| chars[1..end].iter().collect())
}

/// Print raw-fallback snippets grouped by the actual RawTex commands in the AST.
fn print_raw_command_groups(title: &str, snippets: &[String], limit: usize) {
    let mut groups: BTreeMap<String, Vec<&String>> = BTreeMap::new();
    for snippet in snippets {
        let commands = raw_fallback_commands(snippet);
        if commands.is_empty() {
            groups
                .entry("<none>".to_string())
                .or_default()
                .push(snippet);
            continue;
        }
        for command in commands {
            groups.entry(command).or_default().push(snippet);
        }
    }
    let mut groups = groups.into_iter().collect::<Vec<_>>();
    groups.sort_by(|left, right| {
        right
            .1
            .len()
            .cmp(&left.1.len())
            .then_with(|| left.0.cmp(&right.0))
    });
    println!("{title}:");
    for (index, (command, examples)) in groups.iter().enumerate() {
        if index >= limit {
            println!("  ... {} more", groups.len() - limit);
            break;
        }
        if let Some(example) = examples.first() {
            println!("  - \\{command}: {} example={example}", examples.len());
        }
    }
}

/// Print snippets grouped by their source Supported Functions section.
fn print_section_counts(
    title: &str,
    snippets: &[String],
    sections: &BTreeMap<String, String>,
    limit: usize,
) {
    let mut counts = BTreeMap::<String, usize>::new();
    for snippet in snippets {
        let section = sections
            .get(snippet)
            .cloned()
            .unwrap_or_else(|| "<unknown>".to_string());
        *counts.entry(section).or_default() += 1;
    }
    let mut counts = counts.into_iter().collect::<Vec<_>>();
    counts.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    println!("{title}:");
    for (index, (section, count)) in counts.iter().enumerate() {
        if index >= limit {
            println!("  ... {} more", counts.len() - limit);
            break;
        }
        println!("  - {section}: {count}");
    }
}

/// Print remaining raw fallbacks grouped by the blocker that prevents native output.
fn print_remaining_blocker_counts(title: &str, snippets: &[String], limit: usize) {
    let mut groups = BTreeMap::<String, usize>::new();
    for snippet in snippets {
        let commands = raw_fallback_commands(snippet);
        let blocker = remaining_blocker(&commands);
        *groups.entry(blocker.bucket.to_string()).or_default() += 1;
    }
    let mut groups = groups.into_iter().collect::<Vec<_>>();
    groups.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    println!("{title}:");
    for (index, (bucket, count)) in groups.iter().enumerate() {
        if index >= limit {
            println!("  ... {} more", groups.len() - limit);
            break;
        }
        println!("  - {bucket}: {count}");
    }
}

/// Write unclassified complete-formula fallbacks as a reusable probe queue.
fn write_unclassified_jsonl(
    path: &PathBuf,
    snippets: &[String],
    sections: &BTreeMap<String, String>,
) -> Result<(), String> {
    let mut output = String::new();
    for snippet in snippets {
        let section = sections
            .get(snippet)
            .cloned()
            .unwrap_or_else(|| "<unknown>".to_string());
        let commands = raw_fallback_commands(snippet);
        output.push_str(&format!(
            "{{\"section\":\"{}\",\"snippet\":\"{}\",\"commands\":[{}]}}\n",
            json_escape(&section),
            json_escape(snippet),
            commands
                .iter()
                .map(|command| format!("\"{}\"", json_escape(command)))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(path, output).map_err(|err| format!("failed to write {}: {err}", path.display()))
}

/// Write the remaining native-implementation queue with an explicit blocker bucket.
fn write_remaining_jsonl(
    path: &PathBuf,
    snippets: &[String],
    sections: &BTreeMap<String, String>,
) -> Result<(), String> {
    let mut output = String::new();
    for snippet in snippets {
        let section = sections
            .get(snippet)
            .cloned()
            .unwrap_or_else(|| "<unknown>".to_string());
        let commands = raw_fallback_commands(snippet);
        let blocker = remaining_blocker(&commands);
        output.push_str(&format!(
            "{{\"section\":\"{}\",\"snippet\":\"{}\",\"commands\":[{}],\"bucket\":\"{}\",\"note\":\"{}\"}}\n",
            json_escape(&section),
            json_escape(snippet),
            commands
                .iter()
                .map(|command| format!("\"{}\"", json_escape(command)))
                .collect::<Vec<_>>()
                .join(","),
            json_escape(blocker.bucket),
            json_escape(blocker.note)
        ));
    }
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(path, output).map_err(|err| format!("failed to write {}: {err}", path.display()))
}

struct RemainingBlocker {
    bucket: &'static str,
    note: &'static str,
}

/// Explain why a remaining raw fallback is not safe to mark native yet.
fn remaining_blocker(commands: &[String]) -> RemainingBlocker {
    if commands.iter().any(|command| command == "begin") {
        return RemainingBlocker {
            bucket: "unsupported_environment",
            note:
                "CD diagrams need a native arrow-diagram writer or fresh MathType probe evidence.",
        };
    }
    if commands
        .iter()
        .any(|command| command == "mathfrak" || command == "frak")
    {
        return RemainingBlocker {
            bucket: "missing_generated_font_table",
            note: "Fraktur targets exist in the generator but no extractable cached MathType OLE is available.",
        };
    }
    if commands.iter().any(|command| command == "mathsfit") {
        return RemainingBlocker {
            bucket: "missing_font_variant",
            note: "Sans-serif italic needs a verified MathType font/typeface mapping.",
        };
    }
    if commands.iter().any(|command| command == "rule") {
        return RemainingBlocker {
            bucket: "missing_rule_template",
            note: "Visible rule boxes need a documented or probed MTEF representation before native output.",
        };
    }
    if commands.iter().any(|command| command == "tag") {
        return RemainingBlocker {
            bucket: "annotation_semantics",
            note: "Equation tags affect row annotation/numbering semantics and should not be appended as ordinary text.",
        };
    }
    if commands.iter().any(|command| {
        matches!(
            command.as_str(),
            "overgroup" | "undergroup" | "overlinesegment" | "underlinesegment" | "utilde"
        )
    }) {
        return RemainingBlocker {
            bucket: "missing_accent_template_mapping",
            note: "The MTEF template table has no direct selector for this accent family, so probe evidence is required.",
        };
    }
    RemainingBlocker {
        bucket: "needs_probe",
        note: "No safe native mapping has been classified yet.",
    }
}

/// Escape a string for the small JSONL files emitted by this audit tool.
fn json_escape(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            ch if ch.is_control() => format!("\\u{:04x}", ch as u32).chars().collect(),
            ch => vec![ch],
        })
        .collect()
}

/// Print a bounded list without hiding the total count.
fn print_list<'a>(
    title: &str,
    items: impl Iterator<Item = (&'a String, Option<&'a String>)>,
    limit: usize,
) {
    println!("{title}:");
    let mut count = 0usize;
    for (index, (snippet, err)) in items.enumerate() {
        count += 1;
        if index < limit {
            if let Some(err) = err {
                println!("  - {snippet} => {err}");
            } else {
                println!("  - {snippet}");
            }
        }
    }
    if count > limit {
        println!("  ... {} more", count - limit);
    }
}
