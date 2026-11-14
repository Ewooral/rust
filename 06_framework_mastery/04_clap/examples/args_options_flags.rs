//! Arguments, options and flags in more detail: counting flags (`-vvv`),
//! repeated and comma-separated values, `conflicts_with`, `requires`, and
//! an argument group ("exactly one of these").
//! Run: cargo run -p clap_cli --example args_options_flags

use clap::{ArgAction, Args, Parser};
use std::path::PathBuf;

/// Copies files somewhere (pretend)
#[derive(Parser, Debug)]
#[command(name = "copy")]
struct Copy {
    /// Files to copy: one or more POSITIONAL values
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// More detail; repeat for even more (-v, -vv, -vvv)
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,

    /// Say nothing at all
    #[arg(short, long, conflicts_with = "verbose")]
    quiet: bool,

    /// Extensions to skip, comma-separated: --skip tmp,bak
    #[arg(long, value_delimiter = ',')]
    skip: Vec<String>,

    /// Write a log file (needs --log-level)
    #[arg(long, requires = "log_level")]
    log: Option<PathBuf>,

    /// How much the log file gets
    #[arg(long)]
    log_level: Option<String>,

    #[command(flatten)]
    target: Target,
}

/// `#[group(required = true, multiple = false)]`: exactly one of the
/// fields must be given. `#[command(flatten)]` pulls them into `Copy`.
#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct Target {
    /// Copy into this folder
    #[arg(long)]
    to: Option<PathBuf>,

    /// Copy to the clipboard instead
    #[arg(long)]
    clipboard: bool,
}

fn show(args: &[&str]) {
    println!("$ {}", args.join(" "));
    match Copy::try_parse_from(args) {
        Ok(c) => println!("  {c:?}"),
        Err(e) => print!("{}", e.render()),
    }
}

fn main() {
    show(&["copy", "a.txt", "b.txt", "--to", "backup"]);
    show(&[
        "copy",
        "a.txt",
        "-vvv",
        "--skip",
        "tmp,bak",
        "--skip",
        "log",
        "--clipboard",
    ]);
    show(&["copy", "a.txt", "-v", "-q", "--clipboard"]);
    show(&["copy", "a.txt", "--to", "x", "--clipboard"]);
    show(&["copy", "a.txt"]);
    show(&["copy", "a.txt", "--to", "x", "--log", "copy.log"]);
}
