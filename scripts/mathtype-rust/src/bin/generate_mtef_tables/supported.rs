use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use super::ast::Expr;
use super::parser::{normalize_latex, Parser};
use super::raw_fallback::is_known_mathtype_raw_command;

#[derive(Clone)]
pub(super) struct SupportedFunctionsConfig {
    pub(super) path: PathBuf,
    pub(super) sections: Vec<String>,
}

pub(super) struct SupportedCommandCandidate {
    pub(super) command: String,
    pub(super) target_name: String,
}

pub(super) struct SupportedSnippetCandidate {
    pub(super) snippet: String,
    pub(super) section: String,
    pub(super) target_name: String,
}

/// Return pruned single-command symbol candidates from Supported Functions.
pub(super) fn command_candidates(
    supported: &SupportedFunctionsConfig,
    is_existing: impl Fn(&str) -> bool,
) -> Result<Vec<SupportedCommandCandidate>, String> {
    let mut targets = Vec::new();
    for command in command_sections(supported)?.into_keys() {
        if is_existing(&command)
            || is_currently_native_command(&command)
            || is_known_mathtype_raw_command(&command)
            || is_unsupported_probe_command(&command)
        {
            continue;
        }
        targets.push(SupportedCommandCandidate {
            target_name: format!("supported_cmd_{}", target_command_name(&command)),
            command,
        });
    }
    Ok(targets)
}

/// Return complete Supported Functions examples that still need MathType behavior probes.
pub(super) fn snippet_candidates(
    supported: &SupportedFunctionsConfig,
) -> Result<Vec<SupportedSnippetCandidate>, String> {
    let mut targets = Vec::new();
    for (snippet, section) in math_snippet_sections(supported)? {
        let normalized = normalize_latex(&snippet);
        let Ok(expr) = Parser::new(&normalized).parse() else {
            continue;
        };
        let raw_commands = raw_commands(&expr);
        if raw_commands.is_empty()
            || raw_commands
                .iter()
                .all(|command| is_known_mathtype_raw_command(command))
        {
            continue;
        }
        targets.push(SupportedSnippetCandidate {
            target_name: snippet_target_name(&section, &snippet),
            snippet,
            section,
        });
    }
    Ok(targets)
}

/// Return manifest section labels keyed by generated snippet target name.
pub(super) fn snippet_candidate_sections(
    supported: &SupportedFunctionsConfig,
) -> Result<BTreeMap<String, String>, String> {
    Ok(snippet_candidates(supported)?
        .into_iter()
        .map(|candidate| (candidate.target_name, candidate.section))
        .collect())
}

/// Return single-command symbol snippets and the section where each first appears.
pub(super) fn command_sections(
    supported: &SupportedFunctionsConfig,
) -> Result<BTreeMap<String, String>, String> {
    let markdown = fs::read_to_string(&supported.path)
        .map_err(|err| format!("failed to read {}: {err}", supported.path.display()))?;
    let commands = extract_single_command_sections(&markdown, &supported.sections);
    if commands.is_empty() {
        if !supported.sections.is_empty() {
            return Err(format!(
                "no single-command Supported Functions snippets matched sections: {}",
                supported.sections.join(", ")
            ));
        }
    }
    Ok(commands)
}

/// Return complete math snippets and the section where each first appears.
fn math_snippet_sections(
    supported: &SupportedFunctionsConfig,
) -> Result<BTreeMap<String, String>, String> {
    let markdown = fs::read_to_string(&supported.path)
        .map_err(|err| format!("failed to read {}: {err}", supported.path.display()))?;
    Ok(extract_math_snippet_sections(
        &markdown,
        &supported.sections,
    ))
}

/// Extract Markdown math spans as probe-ready complete formulas.
fn extract_math_snippet_sections(
    markdown: &str,
    wanted_sections: &[String],
) -> BTreeMap<String, String> {
    let chars = markdown.chars().collect::<Vec<_>>();
    let sections = section_by_char(markdown);
    let mut snippets = BTreeMap::new();
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
            let section = sections
                .get(start)
                .cloned()
                .unwrap_or_else(|| "<unknown>".to_string());
            if section_is_selected(&section, wanted_sections) {
                if let Some(snippet) =
                    clean_math_snippet(&chars[start..end].iter().collect::<String>())
                {
                    snippets.entry(snippet).or_insert(section);
                }
            }
            index = end + delimiter_len;
        } else {
            index += delimiter_len;
        }
    }
    snippets
}

/// Extract symbol-section code spans that are exactly one alphabetic TeX command.
fn extract_single_command_sections(
    markdown: &str,
    wanted_sections: &[String],
) -> BTreeMap<String, String> {
    let mut commands = BTreeMap::new();
    let mut in_symbol_section = false;
    let mut current_section = String::new();
    for line in markdown.lines() {
        if let Some(heading) = markdown_heading(line) {
            in_symbol_section = is_symbol_probe_section(heading);
            current_section = heading.to_string();
        }
        if in_symbol_section && section_is_selected(&current_section, wanted_sections) {
            extract_single_command_spans(line, &current_section, &mut commands);
        }
    }
    commands
}

/// Return true when no section filter is active or this heading was requested.
fn section_is_selected(section: &str, wanted_sections: &[String]) -> bool {
    wanted_sections.is_empty()
        || wanted_sections
            .iter()
            .any(|wanted_section| wanted_section == section)
}

/// Extract one line's inline-code command spans.
fn extract_single_command_spans(
    line: &str,
    section: &str,
    commands: &mut BTreeMap<String, String>,
) {
    let mut chars = line.chars().peekable();
    let mut in_code = false;
    let mut current = String::new();
    while let Some(ch) = chars.next() {
        if ch == '\u{60}' {
            if chars.peek() == Some(&'\u{60}') {
                continue;
            }
            if in_code {
                if let Some(command) = single_control_word(current.trim()) {
                    commands
                        .entry(command.to_string())
                        .or_insert_with(|| section.to_string());
                }
                current.clear();
            }
            in_code = !in_code;
        } else if in_code {
            current.push(ch);
        }
    }
}

/// Return a Markdown heading's text without leading hashes.
fn markdown_heading(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let level_end = trimmed.chars().take_while(|ch| *ch == '#').count();
    if level_end == 0 || trimmed.as_bytes().get(level_end) != Some(&b' ') {
        return None;
    }
    Some(trimmed[level_end..].trim())
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

/// Skip a Markdown code span so inline examples are not mistaken for math spans.
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

/// Find the end of one Markdown math span while respecting simple brace nesting.
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

/// Keep only complete formulas useful as MathType probe inputs.
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

/// Return true for Supported Functions sections that primarily list symbols.
fn is_symbol_probe_section(heading: &str) -> bool {
    matches!(
        heading,
        "Delimiters"
            | "Letters and Unicode"
            | "Greek Letters"
            | "Other Letters"
            | "Big Operators"
            | "Logic and Set Theory"
            | "Binary Operators"
            | "Relations"
            | "Negated Relations"
            | "Arrows"
            | "Symbols and Punctuation"
    )
}

/// Return a command name for snippets like \alpha, excluding arguments/switches.
fn single_control_word(snippet: &str) -> Option<&str> {
    let body = snippet.strip_prefix('\\')?;
    bare_control_word(body)
}

/// Return the command body when it is a simple alphabetic control word.
fn bare_control_word(body: &str) -> Option<&str> {
    (!body.is_empty() && body.chars().all(|ch| ch.is_ascii_alphabetic())).then_some(body)
}

/// Collect raw command names from parser output for probe-priority filtering.
fn raw_commands(expr: &Expr) -> Vec<String> {
    let mut commands = BTreeMap::<String, ()>::new();
    collect_raw_commands(expr, &mut commands);
    commands.into_keys().collect()
}

/// Walk an expression tree and collect command names preserved as raw TeX.
fn collect_raw_commands(expr: &Expr, commands: &mut BTreeMap<String, ()>) {
    match expr {
        Expr::RawTex(raw) => {
            if let Some(command) = raw.strip_prefix('\\').and_then(bare_control_word) {
                commands.insert(command.to_string(), ());
            }
        }
        Expr::Sequence(items) => {
            for item in items {
                collect_raw_commands(item, commands);
            }
        }
        Expr::Color { content, .. }
        | Expr::Style { content, .. }
        | Expr::Font { content, .. }
        | Expr::Accent { content, .. }
        | Expr::ArrowAccent { content, .. }
        | Expr::BarTemplate { content, .. }
        | Expr::Strike { content, .. }
        | Expr::NotRelation(content)
        | Expr::Boxed(content)
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
            for item in [lower, upper, body].into_iter().flatten() {
                collect_raw_commands(item, commands);
            }
        }
        Expr::Limit { lower, upper, .. } => {
            for item in [lower, upper].into_iter().flatten() {
                collect_raw_commands(item, commands);
            }
        }
        Expr::Brace {
            content,
            annotation,
            ..
        } => {
            collect_raw_commands(content, commands);
            if let Some(annotation) = annotation {
                collect_raw_commands(annotation, commands);
            }
        }
        Expr::Substack { rows }
        | Expr::Subarray { rows, .. }
        | Expr::Matrix { rows, .. }
        | Expr::Environment { rows, .. } => {
            for item in rows.iter().flat_map(|row| row.iter()) {
                collect_raw_commands(item, commands);
            }
        }
        Expr::Char(_)
        | Expr::EmbellishedChar { .. }
        | Expr::CommandSymbol { .. }
        | Expr::BigSymbol(_)
        | Expr::SumOperatorSymbol(_)
        | Expr::Space(_)
        | Expr::FunctionName(_)
        | Expr::Text(_)
        | Expr::Integral { .. } => {}
    }
}

/// Return true when the current parser already handles this command natively.
fn is_currently_native_command(command: &str) -> bool {
    let latex = normalize_latex(&format!("$\\{command}$"));
    Parser::new(&latex)
        .parse()
        .is_ok_and(|expr| !expr.contains_raw_tex())
}

/// Skip known non-symbol commands so doc-derived probes stay focused.
fn is_unsupported_probe_command(command: &str) -> bool {
    matches!(
        command,
        "begin"
            | "end"
            | "left"
            | "middle"
            | "right"
            | "color"
            | "text"
            | "html"
            | "char"
            | "includegraphics"
            | "image"
            | "def"
            | "gdef"
            | "edef"
            | "let"
            | "futurelet"
            | "global"
            | "expandafter"
            | "newcommand"
            | "renewcommand"
            | "providecommand"
    )
}

/// Return a stable target name for a complete snippet probe.
fn snippet_target_name(section: &str, snippet: &str) -> String {
    format!(
        "supported_snippet_{}_{}",
        target_command_name(section).trim_matches('_'),
        stable_hash(snippet)
    )
}

/// Hash snippets to compact names without adding a dependency for manifest-only probes.
fn stable_hash(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

/// Encode command names into Windows-stable target names without case collisions.
fn target_command_name(command: &str) -> String {
    let mut name = String::new();
    for ch in command.chars() {
        if ch.is_ascii_uppercase() {
            name.push_str("_uc_");
            name.push(ch.to_ascii_lowercase());
        } else if ch.is_ascii_alphanumeric() {
            name.push(ch);
        } else {
            name.push('_');
        }
    }
    name
}
