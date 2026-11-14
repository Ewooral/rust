//! Exercise 1: make it compile (E0599: no associated function or constant
//! named `parse` found for struct `Cli`).
//!
//! `#[derive(clap::Parser)]` implements the `Parser` trait for `Cli`, and
//! `parse` is a method OF THAT TRAIT. In Rust you can only call a trait's
//! methods when the trait is in scope, and the derive here is written with
//! its full path, so nothing imported `Parser`.
//!
//! Read the compiler's `help:` lines: it names the exact `use` to add. Then
//! add it (and you may shorten the derive to `#[derive(Parser, Debug)]`).
//!
//! Run: cargo run -p clap_cli --features exercises --bin ex01_fix_parser_not_in_scope -- hello brave new world
//!      cargo run -p clap_cli --features exercises --bin ex01_fix_parser_not_in_scope -- --chars hello

/// Counts words or characters
#[derive(clap::Parser, Debug)]
#[command(name = "count")]
struct Cli {
    /// The words to count
    words: Vec<String>,

    /// Count characters instead of words
    #[arg(short, long)]
    chars: bool,
}

fn main() {
    let cli = Cli::parse();
    let count = if cli.chars {
        cli.words.iter().map(|w| w.chars().count()).sum()
    } else {
        cli.words.len()
    };
    println!("{count}");
}
