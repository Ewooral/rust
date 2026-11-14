//! Solution to exercise 2: make it compile (E0277: the trait bound `Format: Clone` is not
//! satisfied, then E0599: the method `value_parser` exists ... but its
//! trait bounds were not satisfied).
//!
//! Clap must turn the text after `--format` into a `Format`. For an enum of
//! plain choices, `#[derive(ValueEnum)]` does that: each variant becomes an
//! allowed value (`text`, `json`, `csv`) and clap lists them in the help and
//! in error messages. `Format` forgot the derive, so clap can't find any
//! way to parse it, and the message is one of clap's least friendly.
//!
//! 1. Run it and find the line the error points at.
//! 2. Add `ValueEnum` to `Format`'s derives (and `Clone`: clap stores the
//!    parsed value and hands you a copy), and import it from clap.
//! 3. Run it with `-- --format json` and with `-- --format xml`.
//!
//! Run: cargo run -p clap_cli --features solutions --bin sol02_fix_missing_value_enum -- --format json

use clap::{Parser, ValueEnum};

// ValueEnum: the variants become the allowed values text, json, csv.
#[derive(Debug, Clone, ValueEnum)]
enum Format {
    Text,
    Json,
    Csv,
}

/// Prints a tiny report
#[derive(Parser, Debug)]
#[command(name = "report")]
struct Cli {
    /// How to print the report
    #[arg(long, default_value = "text")]
    format: Format,
}

fn main() {
    let cli = Cli::parse();
    match cli.format {
        Format::Text => println!("total: 3 apples"),
        Format::Json => println!("{{\"total\": 3, \"item\": \"apples\"}}"),
        Format::Csv => println!("item,total\napples,3"),
    }
}
