mod ast;
#[cfg(test)]
mod cfb;
mod cli;
mod generated;
mod mtef;
mod ole;
mod parser;
mod typeface;

#[cfg(test)]
mod tests;

/// Start the CLI converter and report any user-facing error string.
fn main() -> Result<(), String> {
    cli::run()
}
