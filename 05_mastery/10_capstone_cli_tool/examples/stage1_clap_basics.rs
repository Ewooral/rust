//! Stage 1: describe the command line as a struct, and let clap's derive
//! macro write the parser, the --help text and the error messages.
//! Run: cargo run -p capstone_cli_tool --example stage1_clap_basics
//! Then try your own arguments:
//!      cargo run -p capstone_cli_tool --example stage1_clap_basics -- app.log -n 3 --json
//!      cargo run -p capstone_cli_tool --example stage1_clap_basics -- --help
//!
//! Lesson 06/04 covers clap in depth (subcommands, validation, completions).

use std::path::PathBuf;

use clap::Parser;

/// A tiny logsum: just the arguments, no work.
#[derive(Debug, Parser)]
#[command(name = "mini", version)]
struct Args {
    /// Files to read (a positional argument: no dashes)
    files: Vec<PathBuf>,

    /// How many top messages to show
    //  `Option`: None when the flag is not typed. Settings that a config file
    //  could also set should be Options, so you can tell "not given" apart.
    #[arg(short = 'n', long)]
    top: Option<usize>,

    /// Print JSON (a flag: present = true)
    #[arg(long)]
    json: bool,
}

fn main() {
    // With real arguments, behave like a real program: parse() prints any
    // error (or the help) and exits by itself.
    if std::env::args().len() > 1 {
        let args = Args::parse();
        println!("parsed: {args:?}");
        return;
    }

    // Without arguments, show several pretend command lines. try_parse_from
    // returns a Result instead of exiting, which is also how you test a CLI.
    let samples: [&[&str]; 5] = [
        &["mini", "app.log"],
        &["mini", "a.log", "b.log", "-n", "3", "--json"],
        &["mini", "--top=10"],
        &["mini", "--top", "lots"],
        &["mini", "--jsn"],
    ];
    for argv in samples {
        println!("$ {}", argv.join(" "));
        match Args::try_parse_from(argv) {
            Ok(args) => println!("  {args:?}\n"),
            Err(e) => {
                // e.kind() says what went wrong; e.exit_code() is 2 for usage errors.
                let first_line = e.to_string().lines().next().unwrap_or("").to_string();
                println!(
                    "  {:?}, exit code {}: {first_line}\n",
                    e.kind(),
                    e.exit_code()
                );
            }
        }
    }
}
