//! clap_cli: a git-style task tracker (the mini project; the logic is in
//! `src/lib.rs`).
//!
//!   cargo run -p clap_cli -- --help
//!   cargo run -p clap_cli -- add Buy milk -p high -t home
//!   cargo run -p clap_cli -- list
//!   cargo run -p clap_cli -- done 1
//!   cargo run -p clap_cli -- completions powershell
//!
//! Tasks go to `tasks.json` in the current folder, or wherever `--store`
//! or the TASKS_STORE environment variable says.
//!
//! Exit codes: 0 success (also for --help and --version), 1 the command
//! failed (e.g. no such task), 2 the arguments were wrong (clap's choice).

use clap::Parser;
use clap_cli::Cli;
use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    // On bad arguments this prints the error and exits with code 2; for
    // --help and --version it prints them and exits with 0.
    let cli = Cli::parse();

    match clap_cli::run(cli, &mut io::stdout().lock(), &mut io::stderr()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE // 1
        }
    }
}
