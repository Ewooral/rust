//! Turning text into typed values, and rejecting bad input early:
//! `ValueEnum`, numeric ranges, and your own parser functions.
//! Run: cargo run -p clap_cli --example value_parsers

use clap::{Parser, ValueEnum};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "backup")]
struct Backup {
    /// How hard to compress
    #[arg(long, value_enum, default_value_t = Level::Normal)]
    level: Level,

    /// Number of worker threads, 1 to 16
    #[arg(long, default_value_t = 4, value_parser = clap::value_parser!(u8).range(1..=16))]
    jobs: u8,

    /// Run every ..., e.g. 90s, 15m, 2h
    #[arg(long, value_parser = parse_duration)]
    every: Option<Duration>,

    /// Where to send a report: one of a fixed list of strings
    #[arg(long, value_parser = ["email", "slack", "none"], default_value = "none")]
    notify: String,
}

/// `ValueEnum`: each variant is an allowed value, in kebab-case. Doc
/// comments describe the values in `--help`; `#[value(..)]` renames or adds
/// aliases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Level {
    /// Fast, bigger files
    Fast,
    /// A sensible balance
    Normal,
    /// Slow, smallest files
    #[value(alias = "max")]
    Best,
}

/// Any `fn(&str) -> Result<T, E>` is a value parser (E: Display). The
/// error text is shown after clap's own "invalid value" message.
fn parse_duration(s: &str) -> Result<Duration, String> {
    let split = s
        .find(|c: char| !c.is_ascii_digit())
        .ok_or("add a unit: s, m or h")?;
    let (number, unit) = s.split_at(split);
    let n: u64 = number
        .parse()
        .map_err(|_| format!("{number:?} is not a number"))?;
    let seconds = match unit {
        "s" => n,
        "m" => n * 60,
        "h" => n * 3600,
        _ => return Err(format!("unknown unit {unit:?}: use s, m or h")),
    };
    Ok(Duration::from_secs(seconds))
}

fn show(args: &[&str]) {
    println!("$ {}", args.join(" "));
    match Backup::try_parse_from(args) {
        Ok(b) => println!("  {b:?}"),
        Err(e) => print!("{}", e.render()),
    }
}

fn main() {
    show(&["backup"]);
    show(&[
        "backup", "--level", "max", "--jobs", "8", "--every", "15m", "--notify", "slack",
    ]);
    show(&["backup", "--level", "turbo"]);
    show(&["backup", "--jobs", "64"]);
    show(&["backup", "--every", "15"]);
    show(&["backup", "--every", "2d"]);
    show(&["backup", "--notify", "pigeon"]);
}
