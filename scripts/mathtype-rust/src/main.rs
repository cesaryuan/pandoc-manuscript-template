mod ast;
mod cfb;
mod cli;
mod conversion;
mod generated;
mod mathtype_ansi;
mod mathtype_input;
mod mtef;
mod ole;
mod parser;
mod raw_fallback;
mod typeface;

#[cfg(test)]
mod tests;

/// Start the CLI converter and report any user-facing error string.
fn main() -> Result<(), String> {
    cli::run()
}
