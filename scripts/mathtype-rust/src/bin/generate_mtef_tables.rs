#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::thread;
use std::time::{Duration, Instant};

#[path = "../ast.rs"]
mod ast;
#[path = "../cfb.rs"]
mod cfb;
#[path = "generate_mtef_tables/format.rs"]
mod format;
#[path = "../generated/mod.rs"]
mod generated;
#[path = "../parser.rs"]
mod parser;
#[path = "../raw_fallback.rs"]
mod raw_fallback;
#[path = "generate_mtef_tables/render.rs"]
mod render;
#[path = "generate_mtef_tables/supported.rs"]
mod supported;
#[path = "generate_mtef_tables/symbol_aliases.rs"]
mod symbol_aliases;
#[path = "../typeface.rs"]
mod typeface;
#[path = "generate_mtef_tables/types.rs"]
mod types;
use format::{json_escape, path_string};
use render::{
    delimiter_command_aliases, render_tables, tex_command_aliases, tex_command_from_formula,
};
use supported::SupportedFunctionsConfig;
use symbol_aliases::{
    SUPPORTED_SYMBOL_ALIASES, SUPPORTED_SYMBOL_SEQUENCE_ALIASES, SUPPORTED_TEXT_ALIASES,
};
use typeface::*;
use types::{category_name, Category, CharRecord, ExplicitFont, Selector, Target};

/// Generate MathType-derived Rust tables for the writer.
fn main() -> Result<(), String> {
    let config = Config::parse(env::args().skip(1).collect())?;
    fs::create_dir_all(&config.work_dir)
        .map_err(|err| format!("failed to create {}: {err}", config.work_dir.display()))?;
    if config.probe_manifest.is_some() {
        return write_probe_manifest(&config);
    }
    if config.supported_snippets {
        return Err(
            "--supported-snippets is manifest-only; pass --write-probe-manifest".to_string(),
        );
    }
    if config.only.is_some() && !config.report_existing && !config.output_explicit {
        return Err(
            "--only generation requires --output to avoid replacing the full generated table"
                .to_string(),
        );
    }
    if config
        .supported_functions
        .as_ref()
        .is_some_and(|supported| !supported.sections.is_empty())
        && !config.report_existing
        && !config.output_explicit
    {
        return Err(
            "--supported-section generation requires --output because it is a partial table probe"
                .to_string(),
        );
    }
    if config.report_existing {
        return report_existing_targets(&config);
    }

    let mut rows = Vec::new();
    for target in config.selected_targets()? {
        match generate_target_row(&config, target) {
            Ok(row) => rows.push(row),
            Err(err) if config.skip_invalid => eprintln!("skipped {}: {err}", target.name),
            Err(err) => return Err(err),
        }
    }

    let source = render_tables(&rows);
    if let Some(parent) = config.output.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(&config.output, source)
        .map_err(|err| format!("failed to write {}: {err}", config.output.display()))?;
    println!("wrote {}", config.output.display());
    Ok(())
}

/// Generate or reuse one MathType probe row.
fn generate_target_row(config: &Config, target: Target) -> Result<(Target, CharRecord), String> {
    let ole_path = config.work_dir.join(format!("{}.ole.bin", target.name));
    let tex_path = config.work_dir.join(format!("{}.tex", target.name));
    fs::write(&tex_path, target.formula)
        .map_err(|err| format!("failed to write {}: {err}", tex_path.display()))?;
    let record = if config.reuse_existing && ole_path.exists() {
        match extract_target_record(&ole_path, target) {
            Ok(record) => record,
            Err(_) => {
                if config.existing_only {
                    return Err(format!("cached OLE is invalid: {}", ole_path.display()));
                }
                run_mathtype_helper(
                    &config.helper,
                    &config.pre_verb,
                    &tex_path,
                    &ole_path,
                    config.timeout_ms,
                )?;
                extract_target_record(&ole_path, target)?
            }
        }
    } else if config.existing_only {
        return Err(format!("cached OLE is missing: {}", ole_path.display()));
    } else {
        run_mathtype_helper(
            &config.helper,
            &config.pre_verb,
            &tex_path,
            &ole_path,
            config.timeout_ms,
        )?;
        extract_target_record(&ole_path, target)?
    };
    Ok((target, record))
}

struct Config {
    helper: PathBuf,
    output: PathBuf,
    work_dir: PathBuf,
    pre_verb: String,
    reuse_existing: bool,
    report_existing: bool,
    existing_only: bool,
    timeout_ms: u64,
    only: Option<String>,
    output_explicit: bool,
    supported_functions: Option<SupportedFunctionsConfig>,
    supported_snippets: bool,
    skip_invalid: bool,
    probe_manifest: Option<PathBuf>,
}

impl Config {
    /// Parse generator options while keeping defaults relative to the crate root.
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut helper = PathBuf::from(
            r"..\..\src\pandoc_manuscript\mathtype\ole_helper\bin\Release\net48\MathTypeOleHelper.exe",
        );
        let mut output = PathBuf::from(r"src\generated\char_tables.rs");
        let mut work_dir = PathBuf::from(r".pmt\mtef-table-generation");
        let mut pre_verb = "2".to_string();
        let mut reuse_existing = false;
        let mut report_existing = false;
        let mut existing_only = false;
        let mut timeout_ms = 30_000u64;
        let mut only = None;
        let mut output_explicit = false;
        let mut supported_functions = None;
        let mut supported_sections = Vec::new();
        let mut supported_snippets = false;
        let mut skip_invalid = false;
        let mut probe_manifest = None;

        let mut index = 0;
        while index < args.len() {
            match args[index].as_str() {
                "--helper" => {
                    index += 1;
                    helper = PathBuf::from(
                        args.get(index)
                            .ok_or_else(|| "--helper requires a path".to_string())?,
                    );
                }
                "--output" => {
                    index += 1;
                    output = PathBuf::from(
                        args.get(index)
                            .ok_or_else(|| "--output requires a path".to_string())?,
                    );
                    output_explicit = true;
                }
                "--work-dir" => {
                    index += 1;
                    work_dir = PathBuf::from(
                        args.get(index)
                            .ok_or_else(|| "--work-dir requires a path".to_string())?,
                    );
                }
                "--pre-verb" => {
                    index += 1;
                    pre_verb = args
                        .get(index)
                        .ok_or_else(|| "--pre-verb requires an OLE verb number".to_string())?
                        .clone();
                }
                "--reuse-existing" => reuse_existing = true,
                "--existing-only" => {
                    reuse_existing = true;
                    existing_only = true;
                }
                "--report-existing" => report_existing = true,
                "--timeout-ms" => {
                    index += 1;
                    let raw = args
                        .get(index)
                        .ok_or_else(|| "--timeout-ms requires a number".to_string())?;
                    timeout_ms = raw
                        .parse::<u64>()
                        .map_err(|err| format!("invalid --timeout-ms {raw}: {err}"))?;
                }
                "--only" => {
                    index += 1;
                    only = Some(
                        args.get(index)
                            .ok_or_else(|| "--only requires a target-name substring".to_string())?
                            .clone(),
                    );
                }
                "--supported-functions" => {
                    index += 1;
                    supported_functions =
                        Some(PathBuf::from(args.get(index).ok_or_else(|| {
                            "--supported-functions requires a path".to_string()
                        })?));
                }
                "--supported-section" => {
                    index += 1;
                    supported_sections.push(
                        args.get(index)
                            .ok_or_else(|| "--supported-section requires a heading".to_string())?
                            .clone(),
                    );
                }
                "--supported-snippets" => supported_snippets = true,
                "--write-probe-manifest" => {
                    index += 1;
                    probe_manifest =
                        Some(PathBuf::from(args.get(index).ok_or_else(|| {
                            "--write-probe-manifest requires a path".to_string()
                        })?));
                }
                "--skip-invalid" => skip_invalid = true,
                other => return Err(format!("unknown option: {other}")),
            }
            index += 1;
        }
        let supported_functions = match (supported_functions, supported_sections) {
            (Some(path), sections) => Some(SupportedFunctionsConfig { path, sections }),
            (None, sections) if !sections.is_empty() => Some(SupportedFunctionsConfig {
                path: PathBuf::from(r"docs\Supported Functions.md"),
                sections,
            }),
            (None, _) => None,
        };

        Ok(Self {
            helper,
            output,
            work_dir,
            pre_verb,
            reuse_existing,
            report_existing,
            existing_only,
            timeout_ms,
            only,
            output_explicit,
            supported_functions,
            supported_snippets,
            skip_invalid,
            probe_manifest,
        })
    }

    /// Return all generated-table targets after applying the optional name filter.
    fn selected_targets(&self) -> Result<Vec<Target>, String> {
        let targets = build_targets(self.supported_functions.as_ref(), self.supported_snippets)?;
        let Some(only) = &self.only else {
            return Ok(targets);
        };
        let filtered = targets
            .into_iter()
            .filter(|target| target.name.contains(only))
            .collect::<Vec<_>>();
        if filtered.is_empty() {
            if self.supported_functions.is_some() && only.starts_with("supported_cmd_") {
                return Ok(filtered);
            }
            return Err(format!("--only matched no generated-table targets: {only}"));
        }
        Ok(filtered)
    }
}

/// Report which target OLE files can be reused without invoking MathType.
fn report_existing_targets(config: &Config) -> Result<(), String> {
    let mut extractable = 0usize;
    let mut missing = Vec::new();
    let mut invalid = Vec::new();
    let targets = config.selected_targets()?;
    for target in &targets {
        let ole_path = config.work_dir.join(format!("{}.ole.bin", target.name));
        if !ole_path.exists() {
            missing.push(*target);
            continue;
        }
        match extract_target_record(&ole_path, *target) {
            Ok(_) => extractable += 1,
            Err(err) => invalid.push((*target, err)),
        }
    }

    println!("target_count={}", targets.len());
    println!("extractable_existing={extractable}");
    println!("missing_existing={}", missing.len());
    println!("invalid_existing={}", invalid.len());
    if let Some(supported) = &config.supported_functions {
        print_supported_candidate_sections(&targets, supported)?;
    }
    if !missing.is_empty() {
        println!("missing_targets:");
        for target in missing {
            println!("  - {} {}", target.name, target.formula);
        }
    }
    if !invalid.is_empty() {
        println!("invalid_targets:");
        for (target, err) in invalid {
            println!("  - {} {} => {}", target.name, target.formula, err);
        }
    }
    Ok(())
}

/// Write a JSONL manifest of selected probe targets without invoking MathType.
fn write_probe_manifest(config: &Config) -> Result<(), String> {
    let output = config
        .probe_manifest
        .as_ref()
        .ok_or_else(|| "probe manifest path is missing".to_string())?;
    let targets = config.selected_targets()?;
    let command_sections = if let Some(supported) = &config.supported_functions {
        if config.supported_snippets {
            BTreeMap::new()
        } else {
            supported::command_sections(supported)?
        }
    } else {
        BTreeMap::new()
    };
    let snippet_sections = if let Some(supported) = &config.supported_functions {
        if config.supported_snippets {
            supported::snippet_candidate_sections(supported)?
        } else {
            BTreeMap::new()
        }
    } else {
        BTreeMap::new()
    };
    let mut lines = String::new();
    for target in &targets {
        let ole_path = config.work_dir.join(format!("{}.ole.bin", target.name));
        let tex_path = config.work_dir.join(format!("{}.tex", target.name));
        // Manifest rows are also runnable probe inputs, so keep the .tex cache materialized.
        fs::write(&tex_path, target.formula)
            .map_err(|err| format!("failed to write {}: {err}", tex_path.display()))?;
        let cache_status = probe_manifest_cache_status(&ole_path, *target);
        let section = if target.category == Category::SupportedSnippet {
            snippet_sections
                .get(target.name)
                .map(String::as_str)
                .unwrap_or("")
        } else {
            tex_command_from_formula(target.formula)
                .and_then(|command| command_sections.get(command))
                .map(String::as_str)
                .unwrap_or("")
        };
        lines.push_str(&format!(
            "{{\"target\":\"{}\",\"category\":\"{}\",\"section\":\"{}\",\"formula\":\"{}\",\"tex_path\":\"{}\",\"ole_path\":\"{}\",\"cache_status\":\"{}\"}}\n",
            json_escape(target.name),
            category_name(target.category),
            json_escape(section),
            json_escape(target.formula),
            json_escape(&path_string(&tex_path)),
            json_escape(&path_string(&ole_path)),
            cache_status
        ));
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(output, lines)
        .map_err(|err| format!("failed to write {}: {err}", output.display()))?;
    println!("wrote_probe_manifest={}", output.display());
    println!("probe_manifest_targets={}", targets.len());
    Ok(())
}

/// Return cache status for manifest rows without forcing snippet probes into table extraction.
fn probe_manifest_cache_status(ole_path: &Path, target: Target) -> &'static str {
    if !ole_path.exists() {
        return "missing";
    }
    if target.category == Category::SupportedSnippet {
        return "present";
    }
    if extract_target_record(ole_path, target).is_ok() {
        "extractable"
    } else {
        "invalid"
    }
}

/// Print section counts for doc-derived Supported Functions probe targets.
fn print_supported_candidate_sections(
    targets: &[Target],
    supported: &SupportedFunctionsConfig,
) -> Result<(), String> {
    let command_sections = supported::command_sections(supported)?;
    let mut sections: BTreeMap<String, usize> = BTreeMap::new();
    for target in targets {
        if !target.name.starts_with("supported_cmd_") {
            continue;
        }
        let Some(command) = tex_command_from_formula(target.formula) else {
            continue;
        };
        let section = command_sections
            .get(command)
            .cloned()
            .unwrap_or_else(|| "<unknown>".to_string());
        *sections.entry(section).or_default() += 1;
    }
    if !sections.is_empty() {
        println!("supported_candidate_sections:");
        for (section, count) in sections {
            println!("  - {section}: {count}");
        }
    }
    Ok(())
}

/// Build every probe formula whose output is an encoding table entry.
fn build_targets(
    supported_functions: Option<&SupportedFunctionsConfig>,
    supported_snippets: bool,
) -> Result<Vec<Target>, String> {
    let mut targets = Vec::new();
    if supported_snippets {
        let supported = supported_functions.ok_or_else(|| {
            "--supported-snippets requires --supported-functions or --supported-section".to_string()
        })?;
        targets.extend(supported_function_snippet_targets(supported)?);
        validate_unique_target_names(&targets)?;
        return Ok(targets);
    }

    for ch in 'A'..='Z' {
        let leaked = Box::leak(format!("$\\mathcal{{{ch}}}$").into_boxed_str());
        targets.push(Target {
            name: Box::leak(format!("mathcal_{ch}").into_boxed_str()),
            category: Category::MathCal,
            formula: leaked,
            selector: Selector::FontPos {
                ch,
                typeface: EXPLICIT_FONT_NEG_1,
                font_pos: ch as u8,
            },
        });
    }

    for ch in 'A'..='Z' {
        let leaked = Box::leak(format!("$\\mathbb{{{ch}}}$").into_boxed_str());
        targets.push(Target {
            name: Box::leak(format!("mathbb_{ch}").into_boxed_str()),
            category: Category::MathBb,
            formula: leaked,
            selector: Selector::FontPos {
                ch,
                typeface: EXPLICIT_FONT_NEG_1,
                font_pos: ch as u8,
            },
        });
    }

    for ch in 'A'..='Z' {
        let leaked = Box::leak(format!("$\\mathfrak{{{ch}}}$").into_boxed_str());
        targets.push(Target {
            name: Box::leak(format!("mathfrak_uc_{ch}").into_boxed_str()),
            category: Category::MathFrak,
            formula: leaked,
            selector: Selector::FontPos {
                ch,
                typeface: EXPLICIT_FONT_NEG_1,
                font_pos: ch as u8,
            },
        });
    }

    for ch in 'a'..='z' {
        let leaked = Box::leak(format!("$\\mathfrak{{{ch}}}$").into_boxed_str());
        targets.push(Target {
            name: Box::leak(format!("mathfrak_lc_{ch}").into_boxed_str()),
            category: Category::MathFrak,
            formula: leaked,
            selector: Selector::FontPos {
                ch,
                typeface: EXPLICIT_FONT_NEG_1,
                font_pos: ch as u8,
            },
        });
    }

    for &(logical, tex, typeface, font_pos, name) in special_targets() {
        targets.push(Target {
            name,
            category: Category::Special,
            formula: tex,
            selector: Selector::FontPos {
                ch: logical,
                typeface,
                font_pos,
            },
        });
    }
    for &(logical, tex, name) in generated_special_targets() {
        targets.push(Target {
            name,
            category: Category::Special,
            formula: tex,
            selector: Selector::LastChar { ch: logical },
        });
    }
    for &(logical, tex, name) in generated_command_specific_targets() {
        targets.push(Target {
            name,
            category: Category::CommandSpecific,
            formula: tex,
            selector: Selector::LastChar { ch: logical },
        });
    }
    for &(logical, tex, name) in generated_command_alias_targets() {
        targets.push(Target {
            name,
            category: Category::CommandAlias,
            formula: tex,
            selector: Selector::LastChar { ch: logical },
        });
    }
    for &(logical, tex, name) in generated_sum_operator_alias_targets() {
        targets.push(Target {
            name,
            category: Category::SumOperatorAlias,
            formula: tex,
            selector: Selector::LastChar { ch: logical },
        });
    }

    for ch in ['+', '-', '=', '<', '>'] {
        let leaked = Box::leak(format!("$x{ch}y$").into_boxed_str());
        targets.push(Target {
            name: Box::leak(format!("operator_{}", operator_name(ch)).into_boxed_str()),
            category: Category::Operator,
            formula: leaked,
            selector: Selector::FontPos {
                ch,
                typeface: FN_SYMBOL,
                font_pos: ch as u8,
            },
        });
    }
    targets.push(Target {
        name: "operator_asterisk",
        category: Category::Operator,
        formula: "$x*y$",
        selector: Selector::PlainChar {
            ch: '*',
            typeface: FN_FUNCTION,
            mtcode: 0x002a,
        },
    });

    targets.push(Target {
        name: "bigop_sum",
        category: Category::BigOperator,
        formula: r"$\sum_{i=1}^{N}x_i$",
        selector: Selector::BigOperator { name: "sum" },
    });
    targets.push(Target {
        name: "bigop_product",
        category: Category::BigOperator,
        formula: r"$\prod_{i=1}^{N}x_i$",
        selector: Selector::BigOperator { name: "product" },
    });
    targets.push(Target {
        name: "bigop_coproduct",
        category: Category::BigOperator,
        formula: r"$\coprod_{i=1}^{N}x_i$",
        selector: Selector::NamedMtCode {
            ch: '\u{2210}',
            name: "coproduct",
        },
    });
    targets.push(Target {
        name: "bigop_union",
        category: Category::BigOperator,
        formula: r"$\bigcup_{i=1}^{N}x_i$",
        selector: Selector::NamedMtCode {
            ch: '\u{222a}',
            name: "union",
        },
    });
    targets.push(Target {
        name: "bigop_intersection",
        category: Category::BigOperator,
        formula: r"$\bigcap_{i=1}^{N}x_i$",
        selector: Selector::NamedMtCode {
            ch: '\u{2229}',
            name: "intersection",
        },
    });
    targets.push(Target {
        name: "bigop_integral",
        category: Category::BigOperator,
        formula: r"$\int_{x}y$",
        selector: Selector::BigOperator { name: "integral" },
    });
    targets.push(Target {
        name: "bigop_contour_loop",
        category: Category::BigOperator,
        formula: r"$\oint_{x}y$",
        selector: Selector::NamedMtCode {
            ch: '\u{ee11}',
            name: "contour_loop",
        },
    });

    if let Some(supported) = supported_functions {
        if supported_snippets {
            targets.extend(supported_function_snippet_targets(supported)?);
        } else {
            targets.extend(supported_function_symbol_targets(supported)?);
        }
    }

    validate_unique_target_names(&targets)?;
    Ok(targets)
}

/// Build opt-in probe targets from Supported Functions single-command snippets.
fn supported_function_symbol_targets(
    supported: &SupportedFunctionsConfig,
) -> Result<Vec<Target>, String> {
    let mut targets = Vec::new();
    for candidate in supported::command_candidates(supported, is_existing_tex_command)? {
        let formula = Box::leak(format!("$\\{}$", candidate.command).into_boxed_str());
        let name = Box::leak(candidate.target_name.into_boxed_str());
        targets.push(Target {
            name,
            category: Category::Special,
            formula,
            selector: Selector::LastInferredChar,
        });
    }
    Ok(targets)
}

/// Build manifest-only probes for complete Supported Functions examples needing classification.
fn supported_function_snippet_targets(
    supported: &SupportedFunctionsConfig,
) -> Result<Vec<Target>, String> {
    let mut targets = Vec::new();
    for candidate in supported::snippet_candidates(supported)? {
        let formula = Box::leak(format!("${}$", candidate.snippet).into_boxed_str());
        let name = Box::leak(candidate.target_name.into_boxed_str());
        targets.push(Target {
            name,
            category: Category::SupportedSnippet,
            formula,
            selector: Selector::LastInferredChar,
        });
    }
    Ok(targets)
}

/// Return true when a command is already covered by generated/static aliases.
fn is_existing_tex_command(command: &str) -> bool {
    special_targets()
        .iter()
        .any(|(_, formula, _, _, _)| tex_command_from_formula(formula) == Some(command))
        || generated_special_targets()
            .iter()
            .any(|(_, formula, _)| tex_command_from_formula(formula) == Some(command))
        || tex_command_aliases()
            .iter()
            .any(|(alias, _)| *alias == command)
        || SUPPORTED_SYMBOL_ALIASES
            .iter()
            .any(|(alias, _)| *alias == command)
        || SUPPORTED_SYMBOL_SEQUENCE_ALIASES
            .iter()
            .any(|(alias, _)| *alias == command)
        || SUPPORTED_TEXT_ALIASES
            .iter()
            .any(|(alias, _)| *alias == command)
        || generated_command_alias_targets()
            .iter()
            .any(|(_, formula, _)| tex_command_from_formula(formula) == Some(command))
        || generated_sum_operator_alias_targets()
            .iter()
            .any(|(_, formula, _)| tex_command_from_formula(formula) == Some(command))
        || generated_command_specific_targets()
            .iter()
            .any(|(_, formula, _)| tex_command_from_formula(formula) == Some(command))
        || delimiter_command_aliases()
            .iter()
            .any(|(alias, _)| *alias == command)
}

/// Reject names that collide on Windows case-insensitive filesystems.
fn validate_unique_target_names(targets: &[Target]) -> Result<(), String> {
    let mut names = BTreeSet::new();
    for target in targets {
        let folded = target.name.to_ascii_lowercase();
        if !names.insert(folded) {
            return Err(format!(
                "target name collides after case folding: {}",
                target.name
            ));
        }
    }
    Ok(())
}

/// Return parser aliases whose output is verified by MathType probes.
fn generated_command_alias_targets() -> &'static [(char, &'static str, &'static str)] {
    &[
        ('\u{2295}', r"$\bigoplus$", "alias_bigoplus"),
        ('\u{2297}', r"$\bigotimes$", "alias_bigotimes"),
        ('\u{2299}', r"$\bigodot$", "alias_bigodot"),
        ('\u{228e}', r"$\biguplus$", "alias_biguplus"),
    ]
}

/// Return command-specific symbols whose CHAR encoding differs from the generic character.
fn generated_command_specific_targets() -> &'static [(char, &'static str, &'static str)] {
    &[('\u{22c5}', r"$\centerdot$", "command_centerdot")]
}

/// Return tmSUMOP-style big symbols verified by MathType probes.
fn generated_sum_operator_alias_targets() -> &'static [(char, &'static str, &'static str)] {
    &[('\u{2294}', r"$\bigsqcup$", "alias_bigsqcup")]
}

/// Return symbol probes whose typeface/font-position should be learned from MathType.
fn generated_special_targets() -> &'static [(char, &'static str, &'static str)] {
    &[
        ('\u{03b6}', r"$\zeta$", "special_zeta"),
        ('\u{03b7}', r"$\eta$", "special_eta"),
        ('\u{03b8}', r"$\theta$", "special_theta"),
        ('\u{03b9}', r"$\iota$", "special_iota"),
        ('\u{03ba}', r"$\kappa$", "special_kappa"),
        ('\u{03be}', r"$\xi$", "special_xi"),
        ('\u{03bc}', r"$\mu$", "special_mu"),
        ('\u{03bd}', r"$\nu$", "special_nu"),
        ('\u{03c3}', r"$\sigma$", "special_sigma"),
        ('\u{03c4}', r"$\tau$", "special_tau"),
        ('\u{03c5}', r"$\upsilon$", "special_upsilon"),
        ('\u{03d5}', r"$\phi$", "special_phi"),
        ('\u{03c8}', r"$\psi$", "special_psi"),
        ('\u{03b5}', r"$\varepsilon$", "special_varepsilon"),
        ('\u{03d1}', r"$\vartheta$", "special_vartheta"),
        ('\u{03d6}', r"$\varpi$", "special_varpi"),
        ('\u{03f1}', r"$\varrho$", "special_varrho"),
        ('\u{03c2}', r"$\varsigma$", "special_varsigma"),
        ('\u{03c6}', r"$\varphi$", "special_varphi"),
        ('\u{0391}', r"$\Alpha$", "special_uc_alpha"),
        ('\u{0392}', r"$\Beta$", "special_uc_beta"),
        ('\u{03a7}', r"$\Chi$", "special_uc_chi"),
        ('\u{0395}', r"$\Epsilon$", "special_uc_epsilon"),
        ('\u{0397}', r"$\Eta$", "special_uc_eta"),
        ('\u{0399}', r"$\Iota$", "special_uc_iota"),
        ('\u{039a}', r"$\Kappa$", "special_uc_kappa"),
        ('\u{039c}', r"$\Mu$", "special_uc_mu"),
        ('\u{039d}', r"$\Nu$", "special_uc_nu"),
        ('\u{03a1}', r"$\Rho$", "special_uc_rho"),
        ('\u{03a4}', r"$\Tau$", "special_uc_tau"),
        ('\u{0398}', r"$\Theta$", "special_uc_theta"),
        ('\u{039e}', r"$\Xi$", "special_uc_xi"),
        ('\u{03a0}', r"$\Pi$", "special_uc_pi"),
        ('\u{03a3}', r"$\Sigma$", "special_uc_sigma"),
        ('\u{03a5}', r"$\Upsilon$", "special_uc_upsilon"),
        ('\u{03a6}', r"$\Phi$", "special_uc_phi"),
        ('\u{03a9}', r"$\Omega$", "special_uc_omega"),
        ('\u{039b}', r"$\Lambda$", "special_uc_lambda"),
        ('\u{0393}', r"$\Gamma$", "special_uc_gamma"),
        ('\u{0396}', r"$\Zeta$", "special_uc_zeta"),
        ('\u{2203}', r"$\exists$", "special_exists"),
        ('\u{2205}', r"$\emptyset$", "special_emptyset"),
        ('\u{2234}', r"$\therefore$", "special_therefore"),
        ('\u{2235}', r"$\because$", "special_because"),
        ('\u{2282}', r"$\subset$", "special_subset"),
        ('\u{2283}', r"$\supset$", "special_supset"),
        ('\u{2286}', r"$\subseteq$", "special_subseteq"),
        ('\u{2287}', r"$\supseteq$", "special_supseteq"),
        ('\u{2209}', r"$\notin$", "special_notin"),
        ('\u{220b}', r"$\ni$", "special_ni"),
        ('\u{00ac}', r"$\neg$", "special_neg"),
        ('\u{2227}', r"$\land$", "special_land"),
        ('\u{2228}', r"$\lor$", "special_lor"),
        ('\u{21a6}', r"$\mapsto$", "special_mapsto"),
        ('\u{2194}', r"$\leftrightarrow$", "special_leftrightarrow"),
        ('&', r"$\And$", "special_ampersand"),
        ('\u{21d0}', r"$\Leftarrow$", "special_double_leftarrow"),
        ('\u{21d2}', r"$\Rightarrow$", "special_double_rightarrow"),
        (
            '\u{21d4}',
            r"$\Leftrightarrow$",
            "special_double_leftrightarrow",
        ),
        ('\u{2191}', r"$\uparrow$", "special_uparrow"),
        ('\u{2193}', r"$\downarrow$", "special_downarrow"),
        ('\u{2195}', r"$\updownarrow$", "special_updownarrow"),
        ('\u{21d1}', r"$\Uparrow$", "special_double_uparrow"),
        ('\u{21d1}', r"$\Uarr$", "special_uarr_alias"),
        ('\u{21d3}', r"$\Downarrow$", "special_double_downarrow"),
        ('\u{21d5}', r"$\Updownarrow$", "special_double_updownarrow"),
        ('\u{2196}', r"$\nwarrow$", "special_nwarrow"),
        ('\u{2197}', r"$\nearrow$", "special_nearrow"),
        ('\u{2199}', r"$\swarrow$", "special_swarrow"),
        ('\u{2198}', r"$\searrow$", "special_searrow"),
        (
            '\u{27f8}',
            r"$\Longleftarrow$",
            "special_long_double_leftarrow",
        ),
        (
            '\u{27fa}',
            r"$\Longleftrightarrow$",
            "special_long_double_leftrightarrow",
        ),
        (
            '\u{27f9}',
            r"$\Longrightarrow$",
            "special_long_double_rightarrow",
        ),
        ('\u{21b0}', r"$\Lsh$", "special_lsh"),
        ('\u{21b1}', r"$\Rsh$", "special_rsh"),
        ('\u{21bc}', r"$\leftharpoonup$", "special_leftharpoonup"),
        ('\u{21c0}', r"$\rightharpoonup$", "special_rightharpoonup"),
        ('\u{2229}', r"$\cap$", "special_cap"),
        ('\u{22d2}', r"$\Cap$", "special_double_cap"),
        ('\u{22d3}', r"$\Cup$", "special_double_cup"),
        ('\u{2213}', r"$\mp$", "special_mp"),
        ('\u{00f7}', r"$\div$", "special_div"),
        ('\u{2296}', r"$\ominus$", "special_ominus"),
        ('\u{2298}', r"$\oslash$", "special_oslash"),
        ('\u{2299}', r"$\odot$", "special_odot"),
        ('\u{228e}', r"$\uplus$", "special_uplus"),
        ('\u{2216}', r"$\setminus$", "special_setminus"),
        ('\u{2216}', r"$\smallsetminus$", "special_smallsetminus"),
        ('\u{2022}', r"$\bullet$", "special_bullet"),
        ('\u{00b7}', r"$\sdot$", "special_sdot"),
        ('\u{2214}', r"$\dotplus$", "special_dotplus"),
        ('\u{2217}', r"$\ast$", "special_ast"),
        ('\u{22c6}', r"$\star$", "special_star"),
        ('\u{25ef}', r"$\bigcirc$", "special_bigcirc"),
        ('\u{25a1}', r"$\Box$", "special_box"),
        ('\u{25c7}', r"$\Diamond$", "special_white_diamond"),
        ('\u{00b6}', r"$\P$", "special_paragraph_sign"),
        ('\u{00a7}', r"$\S$", "special_section_sign"),
        ('\u{2293}', r"$\sqcap$", "special_sqcap"),
        ('\u{2294}', r"$\sqcup$", "special_sqcup"),
        ('\u{229b}', r"$\circledast$", "special_circledast"),
        ('\u{2296}', r"$\circleddash$", "special_circleddash"),
        ('\u{229a}', r"$\circledcirc$", "special_circledcirc"),
        ('\u{2210}', r"$\amalg$", "special_amalg"),
        ('\u{226a}', r"$\ll$", "special_ll"),
        ('\u{226b}', r"$\gg$", "special_gg"),
        ('\u{223c}', r"$\sim$", "special_sim"),
        ('\u{223d}', r"$\backsim$", "special_backsim"),
        ('\u{2243}', r"$\simeq$", "special_simeq"),
        ('\u{22cd}', r"$\backsimeq$", "special_backsimeq"),
        ('\u{2245}', r"$\cong$", "special_cong"),
        ('\u{224d}', r"$\asymp$", "special_asymp"),
        ('\u{224a}', r"$\approxeq$", "special_approxeq"),
        ('\u{224f}', r"$\bumpeq$", "special_bumpeq_lower"),
        ('\u{2250}', r"$\doteq$", "special_doteq"),
        ('\u{2251}', r"$\Doteq$", "special_doteq_dot"),
        ('\u{2252}', r"$\fallingdotseq$", "special_fallingdotseq"),
        ('\u{2253}', r"$\risingdotseq$", "special_risingdotseq"),
        ('\u{2256}', r"$\eqcirc$", "special_eqcirc"),
        ('\u{2257}', r"$\circeq$", "special_circeq"),
        ('\u{224e}', r"$\Bumpeq$", "special_bumpeq"),
        ('\u{22a5}', r"$\perp$", "special_perp"),
        ('\u{2225}', r"$\parallel$", "special_parallel"),
        ('\u{227a}', r"$\prec$", "special_prec"),
        ('\u{227b}', r"$\succ$", "special_succ"),
        ('\u{227c}', r"$\preceq$", "special_preceq"),
        ('\u{227d}', r"$\succeq$", "special_succeq"),
        ('\u{22a8}', r"$\models$", "special_models"),
        ('\u{22a2}', r"$\vdash$", "special_vdash"),
        ('\u{22a3}', r"$\dashv$", "special_dashv"),
        ('\u{22a9}', r"$\Vdash$", "special_vdash_double"),
        ('\u{22aa}', r"$\Vvdash$", "special_vvdash"),
        ('\u{22ac}', r"$\nvdash$", "special_not_vdash"),
        ('\u{22ad}', r"$\nvDash$", "special_not_vdash_double"),
        ('\u{22ae}', r"$\nVdash$", "special_not_vdash_force"),
        ('\u{22af}', r"$\nVDash$", "special_not_vvdash"),
        ('\u{22d0}', r"$\Subset$", "special_double_subset"),
        ('\u{22d1}', r"$\Supset$", "special_double_supset"),
        ('\u{22c8}', r"$\bowtie$", "special_bowtie"),
        ('\u{225c}', r"$\triangleq$", "special_triangleq"),
        ('\u{25bd}', r"$\triangledown$", "special_triangledown"),
        ('\u{22b2}', r"$\triangleleft$", "special_triangleleft"),
        ('\u{22b3}', r"$\triangleright$", "special_triangleright"),
        ('\u{22bc}', r"$\barwedge$", "special_barwedge"),
        ('\u{226c}', r"$\between$", "special_between"),
        ('\u{22ce}', r"$\curlyvee$", "special_curlyvee"),
        ('\u{22cf}', r"$\curlywedge$", "special_curlywedge"),
        ('\u{22c7}', r"$\divideontimes$", "special_divideontimes"),
        ('\u{2306}', r"$\doublebarwedge$", "special_doublebarwedge"),
        ('\u{22ba}', r"$\intercal$", "special_intercal"),
        ('\u{22cb}', r"$\leftthreetimes$", "special_leftthreetimes"),
        ('\u{22d6}', r"$\lessdot$", "special_lessdot"),
        ('\u{22c9}', r"$\ltimes$", "special_ltimes"),
        ('\u{22cc}', r"$\rightthreetimes$", "special_rightthreetimes"),
        ('\u{22ca}', r"$\rtimes$", "special_rtimes"),
        ('\u{25b2}', r"$\blacktriangle$", "special_blacktriangle"),
        (
            '\u{25bc}',
            r"$\blacktriangledown$",
            "special_blacktriangledown",
        ),
        (
            '\u{25c0}',
            r"$\blacktriangleleft$",
            "special_blacktriangleleft",
        ),
        (
            '\u{25b6}',
            r"$\blacktriangleright$",
            "special_blacktriangleright",
        ),
        ('\u{2261}', r"$\equiv$", "special_equiv"),
        ('\u{2192}', r"$\to$", "special_to"),
        ('\u{21da}', r"$\Lleftarrow$", "special_triple_leftarrow"),
        ('\u{21db}', r"$\Rrightarrow$", "special_triple_rightarrow"),
        ('\u{21ba}', r"$\circlearrowleft$", "special_circlearrowleft"),
        (
            '\u{21bb}',
            r"$\circlearrowright$",
            "special_circlearrowright",
        ),
        ('\u{21cd}', r"$\nLeftarrow$", "special_not_double_leftarrow"),
        (
            '\u{21cf}',
            r"$\nRightarrow$",
            "special_not_double_rightarrow",
        ),
        (
            '\u{21ce}',
            r"$\nLeftrightarrow$",
            "special_not_double_leftrightarrow",
        ),
        ('\u{2207}', r"$\nabla$", "special_nabla"),
        ('\u{2200}', r"$\forall$", "special_forall"),
        ('\u{2201}', r"$\complement$", "special_complement"),
        ('\u{220d}', r"$\backepsilon$", "special_backepsilon"),
        ('\u{2135}', r"$\aleph$", "special_aleph"),
        ('\u{2135}', r"$\alef$", "special_alef_alias"),
        ('\u{2135}', r"$\alefsym$", "special_alefsym_alias"),
        ('\u{2136}', r"$\beth$", "special_beth"),
        ('\u{2220}', r"$\angle$", "special_angle"),
        ('\\', r"$\backslash$", "special_backslash"),
        ('\u{2016}', r"$\Vert$", "special_double_vertical_line"),
        ('\u{22a4}', r"$\top$", "special_top"),
        ('\u{22a1}', r"$\boxdot$", "special_boxdot"),
        ('\u{229f}', r"$\boxminus$", "special_boxminus"),
        ('\u{229e}', r"$\boxplus$", "special_boxplus"),
        ('\u{22a0}', r"$\boxtimes$", "special_boxtimes"),
        ('\u{25a1}', r"$\square$", "special_square"),
        ('\u{25b3}', r"$\bigtriangleup$", "special_bigtriangleup"),
        ('\u{25bd}', r"$\bigtriangledown$", "special_bigtriangledown"),
        ('\u{25a0}', r"$\blacksquare$", "special_blacksquare"),
        ('\u{22c4}', r"$\diamond$", "special_diamond"),
        ('\u{2605}', r"$\bigstar$", "special_bigstar"),
        ('\u{24c8}', r"$\circledS$", "special_circled_s"),
        ('\u{29eb}', r"$\blacklozenge$", "special_blacklozenge"),
        ('\u{2663}', r"$\clubsuit$", "special_clubsuit"),
        ('\u{2661}', r"$\heartsuit$", "special_heartsuit"),
        ('\u{2660}', r"$\spadesuit$", "special_spadesuit"),
        ('\u{266d}', r"$\flat$", "special_flat"),
        ('\u{266e}', r"$\natural$", "special_natural"),
        ('\u{266f}', r"$\sharp$", "special_sharp"),
        ('\u{2127}', r"$\mho$", "special_mho"),
        ('\u{2713}', r"$\checkmark$", "special_checkmark"),
        ('\u{2113}', r"$\ell$", "special_ell"),
        ('\u{210f}', r"$\hbar$", "special_hbar"),
        ('\u{0131}', r"$\imath$", "special_imath"),
        ('\u{0237}', r"$\jmath$", "special_jmath"),
        ('\u{00f0}', r"$\eth$", "special_eth"),
        ('\u{03dc}', r"$\digamma$", "special_digamma"),
        ('\u{03f0}', r"$\varkappa$", "special_varkappa"),
        ('\u{2020}', r"$\dagger$", "special_dagger"),
        ('\u{2021}', r"$\ddagger$", "special_ddagger"),
        ('\u{211c}', r"$\Re$", "special_real_part"),
        ('\u{2111}', r"$\Im$", "special_imaginary_part"),
        ('\u{2118}', r"$\wp$", "special_weierstrass_p"),
        ('\u{2141}', r"$\Game$", "special_game"),
        ('\u{2132}', r"$\Finv$", "special_finv"),
        ('\u{2204}', r"$\nexists$", "special_nexists"),
        ('\u{1d55c}', r"$\Bbbk$", "special_blackboard_small_k"),
        ('\u{2295}', r"$\oplus$", "special_oplus"),
        ('\u{2297}', r"$\otimes$", "special_otimes"),
        ('\u{221d}', r"$\propto$", "special_propto"),
        ('\u{2248}', r"$\approx$", "special_approx"),
        ('\u{2202}', r"$\partial$", "special_partial"),
        ('\u{2264}', r"$\le$", "special_le"),
        ('\u{00b1}', r"$\pm$", "special_pm"),
        ('\u{00b0}', r"$\circ$", "special_circ"),
        ('\u{222a}', r"$\cup$", "special_cup"),
        ('\u{221e}', r"$\infin$", "special_infin_alias"),
        ('\u{2026}', r"$\dots$", "special_dots"),
        ('\u{22ef}', r"$\cdots$", "special_cdots"),
        ('\u{22ee}', r"$\vdots$", "special_vdots"),
        ('\u{22f1}', r"$\ddots$", "special_ddots"),
        ('\u{22ef}', r"$\dotsb$", "special_dotsb"),
        ('\u{2035}', r"$\backprime$", "special_backprime"),
    ]
}

/// Return TeX command probes for parser-supported non-ASCII symbols.
fn special_targets() -> &'static [(char, &'static str, u8, u8, &'static str)] {
    &[
        ('\u{03b1}', r"$\alpha$", FN_LC_GREEK, b'a', "special_alpha"),
        ('\u{03b2}', r"$\beta$", FN_LC_GREEK, b'b', "special_beta"),
        ('\u{03b3}', r"$\gamma$", FN_LC_GREEK, b'g', "special_gamma"),
        ('\u{03b4}', r"$\delta$", FN_LC_GREEK, b'd', "special_delta"),
        (
            '\u{03bb}',
            r"$\lambda$",
            FN_LC_GREEK,
            b'l',
            "special_lambda",
        ),
        ('\u{03c0}', r"$\pi$", FN_LC_GREEK, b'p', "special_pi"),
        ('\u{03c1}', r"$\rho$", FN_LC_GREEK, b'r', "special_rho"),
        ('\u{03c7}', r"$\chi$", FN_LC_GREEK, b'c', "special_chi"),
        ('\u{03c9}', r"$\omega$", FN_LC_GREEK, b'w', "special_omega"),
        (
            '\u{0394}',
            r"$\Delta$",
            FN_UC_GREEK,
            b'D',
            "special_uc_delta",
        ),
        ('\u{03a8}', r"$\Psi$", FN_UC_GREEK, b'Y', "special_uc_psi"),
        (
            '\u{03f5}',
            r"$\epsilon$",
            EXPLICIT_FONT_NEG_1,
            0xf2,
            "special_epsilon",
        ),
        ('\u{00d7}', r"$\times$", FN_SYMBOL, 0xb4, "special_times"),
        ('\u{22c5}', r"$\cdot$", FN_SYMBOL, 0xd7, "special_cdot"),
        ('\u{2208}', r"$\in$", FN_SYMBOL, 0xce, "special_in"),
        ('\u{221e}', r"$\infty$", FN_SYMBOL, 0xa5, "special_infty"),
        (
            '\u{2190}',
            r"$\leftarrow$",
            FN_SYMBOL,
            0xac,
            "special_leftarrow",
        ),
        ('\u{2026}', r"$\ldots$", FN_SYMBOL, 0xbc, "special_ldots"),
        ('\u{2260}', r"$\ne$", FN_SYMBOL, 0xb9, "special_ne"),
        ('\u{2265}', r"$\ge$", FN_SYMBOL, 0xb3, "special_ge"),
    ]
}

/// Name ASCII operator probes without punctuation in file names.
fn operator_name(ch: char) -> &'static str {
    match ch {
        '+' => "plus",
        '-' => "minus",
        '=' => "equals",
        '<' => "lt",
        '>' => "gt",
        '*' => "asterisk",
        _ => "unknown",
    }
}

/// Invoke the existing COM helper to let MathType encode one probe formula.
fn run_mathtype_helper(
    helper: &Path,
    pre_verb: &str,
    tex_path: &Path,
    ole_path: &Path,
    timeout_ms: u64,
) -> Result<(), String> {
    let _ = fs::remove_file(ole_path);
    let mut child = Command::new(helper)
        .args([
            "--method",
            "set-data",
            "--pre-verb",
            pre_verb,
            "--format",
            "TeX Input Language",
            "--input",
        ])
        .arg(tex_path)
        .args(["--output"])
        .arg(ole_path)
        .args(["--encoding", "utf16le", "--no-verb"])
        .spawn()
        .map_err(|err| format!("failed to run {}: {err}", helper.display()))?;
    let status = match wait_with_timeout(&mut child, Duration::from_millis(timeout_ms)) {
        Ok(status) => status,
        Err(err) => {
            let _ = fs::remove_file(ole_path);
            return Err(err);
        }
    };
    if !status.success() {
        return Err(format!(
            "{} failed for {} with status {status}",
            helper.display(),
            tex_path.display()
        ));
    }
    Ok(())
}

/// Wait for MathType's COM helper and kill it if one probe hangs.
fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Result<ExitStatus, String> {
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

/// Extract the selected CHAR record from a generated MathType OLE object.
fn extract_target_record(ole_path: &Path, target: Target) -> Result<CharRecord, String> {
    let ole = fs::read(ole_path)
        .map_err(|err| format!("failed to read {}: {err}", ole_path.display()))?;
    let equation_native = cfb::read_regular_stream(&ole, "Equation Native")?;
    let mtef = equation_native
        .get(28..)
        .ok_or_else(|| "Equation Native stream is shorter than the native header".to_string())?;
    let records = collect_char_records(mtef);
    let record = match target.selector {
        Selector::FontPos {
            typeface, font_pos, ..
        } => match target.category {
            Category::MathCal | Category::MathBb | Category::MathFrak => records
                .iter()
                .rev()
                .copied()
                .find(|record| {
                    matches!(
                        record.typeface,
                        EXPLICIT_FONT_NEG_2
                            | EXPLICIT_FONT_NEG_1
                            | FN_VARIABLE
                            | FN_NUMBER
                            | FN_SYMBOL
                            | FN_MT_EXTRA
                    )
                })
                .ok_or_else(|| {
                    format!(
                        "no math-font CHAR record matched {}; records={records:?}",
                        target.name
                    )
                }),
            _ => records
                .iter()
                .rev()
                .copied()
                .find(|record| record.typeface == typeface && record.font_pos == Some(font_pos))
                .ok_or_else(|| format!("no CHAR record matched {}", target.name)),
        },
        Selector::PlainChar {
            typeface, mtcode, ..
        } => records
            .iter()
            .rev()
            .copied()
            .find(|record| {
                record.typeface == typeface && record.mtcode == mtcode && record.font_pos.is_none()
            })
            .ok_or_else(|| format!("no plain CHAR record matched {}", target.name)),
        Selector::NamedMtCode { ch, .. } => records
            .iter()
            .rev()
            .copied()
            .find(|record| record.mtcode == ch as u16)
            .ok_or_else(|| format!("no MTCode CHAR record matched {}", target.name)),
        Selector::LastChar { .. } | Selector::LastInferredChar => records
            .iter()
            .rev()
            .copied()
            .find(|record| !is_probe_placeholder(*record) && (record.options & 0x80) == 0)
            .ok_or_else(|| format!("no visible CHAR record matched {}", target.name)),
        Selector::BigOperator { .. } => records
            .iter()
            .rev()
            .copied()
            .find(|record| {
                record.typeface == 0x86 && record.font_pos.is_some_and(|pos| pos >= 0x80)
            })
            .ok_or_else(|| format!("no big-operator CHAR record matched {}", target.name)),
    }?;
    Ok(normalize_explicit_font_family(
        record,
        mtef,
        target.category,
    ))
}

/// Collect explicit-font CHAR records from MTEF bytes.
fn collect_char_records(mtef: &[u8]) -> Vec<CharRecord> {
    let mut records = Vec::new();
    for index in 0..mtef.len().saturating_sub(4) {
        if mtef[index] != 0x02 {
            continue;
        }
        let options = mtef[index + 1];
        let typeface = mtef[index + 2];
        if !is_supported_typeface(typeface) {
            continue;
        }
        let font_pos = if (options & 0x04) != 0 {
            mtef.get(index + 5).copied()
        } else {
            None
        };
        records.push(CharRecord {
            offset: index,
            options,
            typeface,
            mtcode: u16::from_le_bytes([mtef[index + 3], mtef[index + 4]]),
            font_pos,
            explicit_font: None,
        });
    }
    records
}

/// Normalize standalone explicit-font slots into stable Euclid font families.
fn normalize_explicit_font_family(
    mut record: CharRecord,
    mtef: &[u8],
    category: Category,
) -> CharRecord {
    if category == Category::Special
        && record.typeface == EXPLICIT_FONT_NEG_1
        && record.font_pos.is_some()
    {
        record.explicit_font = match explicit_font_family_before(mtef, record.offset) {
            Some(1) => Some(ExplicitFont::EuclidMathOne),
            Some(2) => Some(ExplicitFont::EuclidMathTwo),
            _ => None,
        };
    }
    record
}

/// Return the latest Euclid Math font family defined before a CHAR record.
fn explicit_font_family_before(mtef: &[u8], offset: usize) -> Option<u8> {
    let prefix = mtef.get(..offset)?;
    let one = find_last_ascii(prefix, b"EuclidMath1\0");
    let two = find_last_ascii(prefix, b"EuclidMath2\0");
    match (one, two) {
        (Some(left), Some(right)) if right > left => Some(2),
        (Some(_), Some(_)) | (Some(_), None) => Some(1),
        (None, Some(_)) => Some(2),
        (None, None) => None,
    }
}

/// Find the last byte offset of a literal ASCII needle.
fn find_last_ascii(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .rposition(|window| window == needle)
}

/// Ignore boilerplate CHAR records MathType wraps around single-symbol probes.
fn is_probe_placeholder(record: CharRecord) -> bool {
    (record.options == 0x02 && record.typeface == FN_FUNCTION && record.mtcode == 0x0002)
        || (record.options == 0x00 && record.typeface == FN_TEXT && record.mtcode == 0x0101)
}

/// Return true for typefaces emitted by the current probe set.
fn is_supported_typeface(typeface: u8) -> bool {
    matches!(
        typeface,
        EXPLICIT_FONT_NEG_2
            | EXPLICIT_FONT_NEG_1
            | FN_FUNCTION
            | FN_VARIABLE
            | FN_LC_GREEK
            | FN_UC_GREEK
            | FN_SYMBOL
            | FN_NUMBER
            | FN_MT_EXTRA
            | FN_TEXT_FE
    )
}
