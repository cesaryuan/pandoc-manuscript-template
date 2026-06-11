mod ast;
mod cli;
mod mtef;
mod ole;
mod parser;

#[cfg(test)]
mod tests;

/// Start the CLI converter and report any user-facing error string.
fn main() -> Result<(), String> {
    cli::run()
}
