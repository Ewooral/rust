//! End-to-end tests: run the real minigrep binary as a separate PROCESS, the
//! way a user's shell would, and check its stdout, stderr and exit code.
//!
//! Cargo builds the binary first and tells us where it is through the
//! CARGO_BIN_EXE_<name> environment variable.
//!
//! Run: cargo test -p minigrep --test cli

use std::path::PathBuf;
use std::process::{Command, Output};

fn poem() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("poem.txt")
}

/// Runs minigrep with the given arguments. IGNORE_CASE is removed from the
/// child's environment unless a test sets it, so the user's own shell
/// settings can't change the results.
fn minigrep(args: &[&str], ignore_case: bool) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_minigrep"));
    cmd.args(args).env_remove("IGNORE_CASE");
    if ignore_case {
        cmd.env("IGNORE_CASE", "1"); // only for the child process
    }
    cmd.output().expect("minigrep should start")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n")
}

#[test]
fn finds_matching_lines() {
    let out = minigrep(&["frog", poem().to_str().unwrap()], false);
    assert!(out.status.success());
    assert_eq!(stdout(&out), "How public, like a frog\n");
}

#[test]
fn case_sensitive_by_default() {
    let out = minigrep(&["to", poem().to_str().unwrap()], false);
    assert_eq!(
        stdout(&out),
        "Are you nobody, too?\nHow dreary to be somebody!\n"
    );
}

#[test]
fn ignore_case_from_the_environment() {
    let out = minigrep(&["to", poem().to_str().unwrap()], true);
    assert_eq!(
        stdout(&out),
        "Are you nobody, too?\nHow dreary to be somebody!\nTo tell your name the livelong day\nTo an admiring bog!\n"
    );
}

#[test]
fn missing_arguments_go_to_stderr_with_exit_code_1() {
    let out = minigrep(&[], false);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty(), "nothing on stdout");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("Problem parsing arguments: not enough arguments"));
}

#[test]
fn missing_file_is_an_application_error() {
    let out = minigrep(&["x", "no_such_file.txt"], false);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).starts_with("Application error:"));
}
