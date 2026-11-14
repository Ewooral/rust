//! Exercise 6: errors, output streams and exit codes, the way well-behaved
//! command-line tools do them.
//!
//! `main_with` is `main` made testable: it gets the arguments and two
//! buffers standing in for stdout and stderr, and returns the exit code.
//! Implement it:
//!
//! 1. Parse with `Cli::try_parse_from(args)`.
//! 2. If that fails with a `clap::Error` `e`: write `e.render()` to `out`
//!    when `e.use_stderr()` is false (that's `--help` and `--version`),
//!    otherwise to `err`; return `e.exit_code()` (as a `u8`). That gives
//!    0 for help/version and 2 for bad arguments, like `Cli::parse()` does.
//! 3. Otherwise call `calculate(cli)`:
//!    `Ok(answer)` -> write the answer and a newline to `out`, return 0;
//!    `Err(message)` -> write `error: <message>` and a newline to `err`,
//!    return 1.
//!
//! Hints: `write!(out, "{}", e.render())` and `writeln!(err, "error: {msg}")`
//! (with `use std::io::Write`); `u8::try_from(code).unwrap_or(1)`.
//!
//! Test: cargo test -p clap_cli --features exercises --bin ex06_exit_codes

#![allow(unused_variables, unused_imports, dead_code)]

use clap::{Parser, Subcommand};
use std::io::Write;
use std::process::ExitCode;

/// A two-operation calculator
#[derive(Parser, Debug)]
#[command(name = "calc", version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    op: Op,
}

#[derive(Subcommand, Debug)]
enum Op {
    /// a + b
    Add { a: f64, b: f64 },
    /// a / b
    Div { a: f64, b: f64 },
}

fn calculate(cli: Cli) -> Result<String, String> {
    match cli.op {
        Op::Add { a, b } => Ok((a + b).to_string()),
        Op::Div { b: 0.0, .. } => Err("cannot divide by zero".to_string()),
        Op::Div { a, b } => Ok((a / b).to_string()),
    }
}

fn main_with(args: &[&str], out: &mut Vec<u8>, err: &mut Vec<u8>) -> u8 {
    todo!()
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = main_with(&args, &mut out, &mut err);
    std::io::stdout().write_all(&out).unwrap();
    std::io::stderr().write_all(&err).unwrap();
    ExitCode::from(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(args: &[&str]) -> (u8, String, String) {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = main_with(args, &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    #[test]
    fn answers_go_to_stdout_with_code_0() {
        assert_eq!(
            run(&["calc", "div", "5", "2"]),
            (0, "2.5\n".into(), String::new())
        );
        assert_eq!(
            run(&["calc", "add", "1", "2"]),
            (0, "3\n".into(), String::new())
        );
    }

    #[test]
    fn help_and_version_are_not_failures() {
        let (code, out, err) = run(&["calc", "--help"]);
        assert_eq!(code, 0);
        assert!(out.contains("Usage: calc <COMMAND>"), "{out}");
        assert!(err.is_empty());
        assert_eq!(
            run(&["calc", "--version"]),
            (0, "calc 1.0.0\n".into(), String::new())
        );
    }

    #[test]
    fn bad_arguments_are_2_and_failures_are_1() {
        let (code, out, err) = run(&["calc", "div", "5", "two"]);
        assert_eq!(code, 2);
        assert!(out.is_empty());
        assert!(
            err.starts_with("error: invalid value 'two' for '<B>'"),
            "{err}"
        );

        assert_eq!(
            run(&["calc", "div", "1", "0"]),
            (1, String::new(), "error: cannot divide by zero\n".into())
        );
    }
}
