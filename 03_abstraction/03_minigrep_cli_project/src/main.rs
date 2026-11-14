//! minigrep: search for a string in a file and print the matching lines.
//!
//! From this folder (03_abstraction/03_minigrep_cli_project):
//!   cargo run -- to poem.txt
//!   cargo run -- body poem.txt
//!   IGNORE_CASE=1 cargo run -- to poem.txt          (bash)
//!   $env:IGNORE_CASE=1; cargo run -- to poem.txt    (PowerShell; then Remove-Item Env:IGNORE_CASE)
//!   cargo run -- to poem.txt > output.txt           (results to a file, errors still on screen)
//!
//! main.rs has only the jobs a library can't do for itself: read the real
//! arguments, and turn errors into a message on STDERR and an exit code.

use std::env;
use std::process;

use minigrep::Config;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        eprintln!("Usage: minigrep <query> <file_path>");
        process::exit(1);
    });

    if let Err(e) = minigrep::run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
