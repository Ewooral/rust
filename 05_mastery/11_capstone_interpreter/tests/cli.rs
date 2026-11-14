//! End-to-end tests of the `capstone_interpreter` binary: run scripts, feed
//! the REPL through a pipe, and check output, errors and exit codes.
//!
//! Run: cargo test -p capstone_interpreter --test cli

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

fn kite() -> Command {
    Command::new(env!("CARGO_BIN_EXE_capstone_interpreter"))
}

fn program(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("programs")
        .join(name)
}

/// A scratch script in Cargo's temp folder for integration tests.
fn script(name: &str, source: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("kite_tests");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, source).unwrap();
    path
}

/// Runs the REPL with `input` piped in (so stdin is not a terminal).
fn repl(input: &str) -> Output {
    let mut child = kite()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("kite should start");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).replace("\r\n", "\n")
}

#[test]
fn runs_a_script_file() {
    let out = kite().arg(program("fib.kite")).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = text(&out.stdout);
    assert!(stdout.starts_with("fib(0) = 0\nfib(1) = 1\n"), "{stdout}");
    assert!(stdout.ends_with("fib_loop(50) = 12586269025\n"), "{stdout}");
}

#[test]
fn fizzbuzz_script() {
    let out = kite().arg(program("fizzbuzz.kite")).output().unwrap();
    let lines: Vec<String> = text(&out.stdout).lines().map(String::from).collect();
    assert_eq!(lines.len(), 15);
    assert_eq!(lines[2], "Fizz");
    assert_eq!(lines[4], "Buzz");
    assert_eq!(lines[14], "FizzBuzz");
}

#[test]
fn the_repl_reads_lines_from_a_pipe() {
    let out = repl(
        "let x = 20;\nx + 1\nfn twice(n) {\n  return n * 2;\n}\ntwice(x)\nprint(\"done\");\n\"text\"\n",
    );
    assert_eq!(out.status.code(), Some(0));
    // Piped input: no banner, no prompts. Values are echoed, strings quoted.
    assert_eq!(text(&out.stdout), "21\n40\ndone\n\"text\"\n");
    assert_eq!(text(&out.stderr), "");
}

#[test]
fn repl_errors_go_to_stderr_and_the_session_continues() {
    let out = repl("1 / 0\n2 + 2\n");
    assert_eq!(text(&out.stdout), "4\n");
    let stderr = text(&out.stderr);
    assert!(
        stderr.starts_with("runtime error at line 1, column 3: division by zero"),
        "{stderr}"
    );
    assert!(stderr.contains("1 | 1 / 0\n  |   ^"), "{stderr}");
    assert_eq!(
        out.status.code(),
        Some(70),
        "a piped session with errors fails"
    );
}

#[test]
fn a_syntax_error_in_a_script_is_exit_code_65_with_a_caret() {
    let path = script("syntax.kite", "let x = 1;\nlet y = (x + ;\n");
    let out = kite().arg(&path).output().unwrap();
    assert_eq!(out.status.code(), Some(65));
    assert!(out.stdout.is_empty());
    assert_eq!(
        text(&out.stderr),
        "syntax error at line 2, column 14: expected an expression, found ';'\n --> line 2, column 14\n  |\n2 | let y = (x + ;\n  |              ^\n"
    );
}

#[test]
fn a_runtime_error_is_exit_code_70_after_earlier_output() {
    let path = script(
        "runtime.kite",
        "print(\"before\");\nprint(missing);\nprint(\"after\");\n",
    );
    let out = kite().arg(&path).output().unwrap();
    assert_eq!(out.status.code(), Some(70));
    assert_eq!(text(&out.stdout), "before\n");
    assert!(
        text(&out.stderr)
            .starts_with("runtime error at line 2, column 7: undefined variable 'missing'")
    );
}

#[test]
fn deep_recursion_works_and_runaway_recursion_is_an_error() {
    // 900 nested calls: fine on the binary's 64 MiB interpreter thread.
    let path = script(
        "deep.kite",
        "fn down(n) { if n == 0 { return 0; } return down(n - 1) + 1; }\nprint(down(900));\n",
    );
    let out = kite().arg(&path).output().unwrap();
    assert_eq!(text(&out.stdout), "900\n", "{}", text(&out.stderr));

    let path = script("forever.kite", "fn f(n) { return f(n + 1); }\nf(0);\n");
    let out = kite().arg(&path).output().unwrap();
    assert_eq!(out.status.code(), Some(70));
    assert!(text(&out.stderr).contains("too much recursion (more than 1000 nested calls)"));
}

#[test]
fn a_missing_file_is_exit_code_66_and_extra_arguments_64() {
    let out = kite().arg("no_such_file.kite").output().unwrap();
    assert_eq!(out.status.code(), Some(66));
    assert!(text(&out.stderr).starts_with("kite: cannot read no_such_file.kite"));

    let out = kite().args(["a.kite", "b.kite"]).output().unwrap();
    assert_eq!(out.status.code(), Some(64));
    assert_eq!(text(&out.stderr), "usage: kite [script.kite]\n");
}
