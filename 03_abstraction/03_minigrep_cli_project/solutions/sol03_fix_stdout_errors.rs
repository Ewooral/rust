//! Solution to exercise 3: errors belong on stderr, with a failure exit code.
//!
//! `report` prints a result and returns the process exit code. Right now it
//! prints the error to STDOUT and returns 0 ("success"). See the problem as a
//! script would, by redirecting stdout to a file:
//!   bash:        cargo run -p minigrep --features solutions --bin sol03_fix_stdout_errors > out.txt; echo $?
//!   PowerShell:  cargo run -p minigrep --features solutions --bin sol03_fix_stdout_errors > out.txt; $LASTEXITCODE
//! The error disappears into out.txt and the exit code says 0. Fix `report`
//! so errors go to `err` and the code is 1. (Then delete out.txt.)
//!
//! `report` writes to whatever streams it is GIVEN, instead of calling
//! println!/eprintln! directly. main passes the real stdout and stderr;
//! the tests pass in byte buffers and check what ended up where.
//!
//! Test: cargo test -p minigrep --features solutions --bin sol03_fix_stdout_errors

use std::io::{self, Write};

fn report(result: io::Result<String>, out: &mut impl Write, err: &mut impl Write) -> i32 {
    match result {
        Ok(text) => {
            writeln!(out, "{text}").expect("write to stdout");
            0
        }
        Err(e) => {
            writeln!(err, "Application error: {e}").expect("write to stderr");
            1
        }
    }
}

fn main() {
    let result = std::fs::read_to_string("definitely_missing_file.txt");
    let code = report(result, &mut io::stdout(), &mut io::stderr());
    std::process::exit(code);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_goes_to_stdout_with_code_0() {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = report(Ok("hello".to_string()), &mut out, &mut err);
        assert_eq!(code, 0);
        assert_eq!(String::from_utf8_lossy(&out), "hello\n");
        assert!(err.is_empty());
    }

    #[test]
    fn errors_go_to_stderr_with_code_1() {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let failure = io::Error::new(io::ErrorKind::NotFound, "no such file");
        let code = report(Err(failure), &mut out, &mut err);
        assert_eq!(code, 1);
        assert!(out.is_empty(), "nothing should go to stdout");
        assert_eq!(
            String::from_utf8_lossy(&err),
            "Application error: no such file\n"
        );
    }
}
