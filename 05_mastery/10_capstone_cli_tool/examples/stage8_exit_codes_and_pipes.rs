//! Stage 8: exit codes, and surviving a closed pipe.
//! Run: cargo run -p capstone_cli_tool --example stage8_exit_codes_and_pipes
//!      cargo run -p capstone_cli_tool --example stage8_exit_codes_and_pipes | head -n 3
//!      cargo run -p capstone_cli_tool --example stage8_exit_codes_and_pipes -- --panic | head -n 3
//! (bash, or Git Bash on Windows: PowerShell's `Select-Object -First 3` behaves differently)
//!
//! It prints 100,000 numbered lines. `head -n 3` reads three and closes the
//! pipe; the next write then fails with BrokenPipe. `println!` PANICS on
//! that error; writing with `writeln!` and treating BrokenPipe as "the
//! reader has had enough" ends quietly with exit code 0.

use std::env;
use std::io::{self, BufWriter, Write};
use std::process::ExitCode;

fn print_lines() -> io::Result<()> {
    let mut out = BufWriter::new(io::stdout().lock());
    for i in 1..=100_000 {
        writeln!(out, "line {i}")?; // returns Err(BrokenPipe) instead of panicking
    }
    out.flush()
}

fn main() -> ExitCode {
    if env::args().any(|a| a == "--panic") {
        // The naive way: panics with "failed printing to stdout" once the
        // pipe closes (try it with `| head -n 3`).
        for i in 1..=100_000 {
            println!("line {i}");
        }
        return ExitCode::SUCCESS;
    }

    match print_lines() {
        Ok(()) => {
            eprintln!("all lines written; exit code 0");
            ExitCode::SUCCESS
        }
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {
            eprintln!("the reader closed the pipe early; that's fine: exit code 0");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            // 0-255 is the portable range (Unix keeps only the lowest byte),
            // so ExitCode::from takes a u8.
            ExitCode::from(2)
        }
    }
}
