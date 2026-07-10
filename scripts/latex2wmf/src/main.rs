//! Cross-platform command-line entry point for formula WMF previews.
//!
//! The program deliberately separates formula layout from WMF serialization:
//! `ratex` parses LaTeX directly, while `typst` first converts LaTeX math with
//! MiTeX and compiles the resulting Typst source through `typst-as-lib`. Both
//! paths emit SVG, which is normalized by `usvg` and translated into the small
//! path-only WMF subset used by formulas. The CLI also writes placement JSON
//! consumed by the Python DOCX integration.

mod cli;
#[cfg(test)]
mod snapshot_tests;
mod svg_backend;
mod wmf;

/// Run the converter and expose a concise error to shell callers.
fn main() -> Result<(), String> {
    cli::run()
}
