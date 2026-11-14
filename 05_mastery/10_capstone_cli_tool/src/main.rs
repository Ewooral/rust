//! logsum: summarise log files. The capstone CLI tool.
//!
//! From this folder (05_mastery/10_capstone_cli_tool):
//!   cargo run -- data/app.log
//!   cargo run -- data/app.log --json
//!   cargo run -- data/app.log --config data/logsum.toml --show-config
//!   cat data/app.log | cargo run                     (standard input)
//!   cargo run -- data/app.log --fail-on error; echo $?
//!   cargo run -- --help
//!
//! main.rs has only the jobs nobody else can do: parse the real arguments,
//! run the app, and turn the result into an exit code and, for errors, a
//! message on stderr.

mod app;
mod cli;

use std::io;
use std::process::ExitCode;

use clap::Parser;

use app::Outcome;
use cli::Cli;

fn main() -> ExitCode {
    // On bad arguments clap prints the error and exits with code 2; on
    // --help or --version it prints them and exits with code 0.
    let cli = Cli::parse();

    match app::run(cli) {
        Ok(Outcome::Clean) => ExitCode::SUCCESS,
        Ok(Outcome::ThresholdReached) => ExitCode::from(1),
        // `logsum big.log | head -n 3`: head closed the pipe early. That's
        // the reader saying "enough", not a failure.
        Err(err) if is_broken_pipe(&err) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("logsum: error: {err}");
            for cause in err.chain().skip(1) {
                eprintln!("  caused by: {cause}");
            }
            ExitCode::from(2)
        }
    }
}

/// Whether any error in the chain is an `io::Error` of kind `BrokenPipe`.
fn is_broken_pipe(err: &anyhow::Error) -> bool {
    err.chain().any(|cause| {
        cause
            .downcast_ref::<io::Error>()
            .is_some_and(|e| e.kind() == io::ErrorKind::BrokenPipe)
    })
}
