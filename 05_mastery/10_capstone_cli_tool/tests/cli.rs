//! End-to-end tests: run the real logsum binary as a separate PROCESS, the
//! way a shell or a CI job would, and check stdout, stderr and the exit code.
//!
//! Cargo builds the binary first and tells us where it is through
//! CARGO_BIN_EXE_<name>. It also gives integration tests a scratch folder,
//! CARGO_TARGET_TMPDIR, for files they create.
//!
//! Run: cargo test -p capstone_cli_tool --test cli

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use serde_json::Value;

fn sample_log() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/app.log")
}

/// A fresh, empty folder for one test (so no logsum.toml is found by accident).
fn scratch_dir(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("logsum_tests")
        .join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// A logsum command with a clean environment: the user's own LOGSUM_* or
/// NO_COLOR variables can't change the results, and it runs in an empty
/// folder. Tests add variables to the CHILD only (never `env::set_var`).
fn logsum(dir: &PathBuf) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_capstone_cli_tool"));
    for var in [
        "LOGSUM_CONFIG",
        "LOGSUM_MIN_LEVEL",
        "LOGSUM_TOP",
        "LOGSUM_COLOR",
        "NO_COLOR",
    ] {
        cmd.env_remove(var);
    }
    cmd.current_dir(dir).stdin(Stdio::null());
    cmd
}

fn run(cmd: &mut Command) -> Output {
    cmd.output().expect("logsum should start")
}

/// Runs with `input` piped into standard input.
fn run_with_stdin(cmd: &mut Command, input: &[u8]) -> Output {
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("logsum should start");
    child.stdin.take().unwrap().write_all(input).unwrap(); // dropped here: end of input
    child.wait_with_output().unwrap()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).replace("\r\n", "\n")
}

#[test]
fn summarises_a_file_as_a_plain_table() {
    let dir = scratch_dir("table");
    let out = run(logsum(&dir).arg(sample_log()));
    assert_eq!(out.status.code(), Some(0));
    let text = stdout(&out);
    assert!(text.starts_with("logsum: 1 input, 29 lines (1.5 KiB), 2 without a level\n"));
    assert!(text.contains("error      5   18.5%\n"), "{text}");
    assert!(
        text.contains("    4  warn   [db] slow query took #ms\n"),
        "{text}"
    );
    // stdout is a pipe here, so: no colours, and no progress line on stderr.
    assert!(!text.contains('\x1b'));
    assert_eq!(stderr(&out), "");
}

#[test]
fn reads_standard_input_when_no_file_or_dash_is_given() {
    let dir = scratch_dir("stdin");
    let log = fs::read(sample_log()).unwrap();
    for args in [&[][..], &["-"][..]] {
        let out = run_with_stdin(logsum(&dir).args(args).arg("--json"), &log);
        assert!(out.status.success());
        let json: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(json["inputs"][0], "-");
        assert_eq!(json["lines"], 29);
    }
}

#[test]
fn json_output_is_complete_and_machine_readable() {
    let dir = scratch_dir("json");
    let out = run(logsum(&dir)
        .arg(sample_log())
        .args(["--json", "--top", "2"]));
    assert!(out.status.success());
    let json: Value = serde_json::from_slice(&out.stdout).expect("valid JSON");
    assert_eq!(json["levels"]["error"], 5);
    assert_eq!(json["levels"]["fatal"], 1);
    assert_eq!(json["unparsed"], 2);
    assert_eq!(json["first_timestamp"], "2026-05-01T09:00:00Z");
    assert_eq!(json["top"].as_array().unwrap().len(), 2);
    assert_eq!(
        json["top"][1]["example"],
        "payment 991 failed: card declined"
    );
}

#[test]
fn several_files_add_up() {
    let dir = scratch_dir("several");
    let out = run(logsum(&dir)
        .arg(sample_log())
        .arg(sample_log())
        .arg("--json"));
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["inputs"].as_array().unwrap().len(), 2);
    assert_eq!(json["lines"], 58);
}

#[test]
fn invalid_utf8_and_crlf_are_handled() {
    let dir = scratch_dir("utf8");
    let input = b"2026-05-01 10:00:00 ERROR bad byte \xFF here\r\nINFO fine\r\n";
    let out = run_with_stdin(logsum(&dir).arg("--json"), input);
    assert!(out.status.success(), "{}", stderr(&out));
    let json: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["lines"], 2);
    assert_eq!(json["top"][0]["example"], "bad byte \u{FFFD} here");
}

#[test]
fn fail_on_sets_exit_code_1_only_when_the_level_is_present() {
    let dir = scratch_dir("fail_on");
    let out = run(logsum(&dir).arg(sample_log()).args(["--fail-on", "error"]));
    assert_eq!(out.status.code(), Some(1));
    assert!(!out.stdout.is_empty(), "the summary is still printed");

    let out = run_with_stdin(
        logsum(&dir).args(["--fail-on", "error"]),
        b"INFO ok\nWARN hmm\n",
    );
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn a_missing_file_is_exit_code_2_with_the_cause_on_stderr() {
    let dir = scratch_dir("missing");
    let out = run(logsum(&dir).arg("no_such.log"));
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty(), "nothing on stdout");
    let err = stderr(&out);
    assert!(
        err.starts_with("logsum: error: cannot open no_such.log\n  caused by: "),
        "{err}"
    );
}

#[test]
fn bad_usage_is_exit_code_2_from_clap() {
    let dir = scratch_dir("usage");
    let out = run(logsum(&dir).args(["--top", "lots"]));
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("invalid value 'lots' for '--top <N>'"));
}

#[test]
fn help_shows_usage_examples_and_exit_codes() {
    let dir = scratch_dir("help");
    let out = run(logsum(&dir).arg("--help"));
    assert_eq!(out.status.code(), Some(0));
    let text = stdout(&out);
    assert!(text.contains("Usage: logsum [OPTIONS] [FILE]..."), "{text}");
    assert!(text.contains("--fail-on <LEVEL>"));
    assert!(text.contains("Exit codes: 0 ok, 1 --fail-on level found, 2 error"));

    let out = run(logsum(&dir).arg("--version"));
    assert_eq!(
        stdout(&out),
        format!("logsum {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn settings_precedence_default_file_env_flag() {
    let dir = scratch_dir("precedence");
    fs::write(
        dir.join("logsum.toml"),
        "min_level = \"error\"\ntop = 3\ncolor = \"never\"\n",
    )
    .unwrap();

    // Only the file (found in the current folder).
    let out = run(logsum(&dir).arg("--show-config"));
    let text = stdout(&out);
    assert!(
        text.contains("min_level = error    from config file"),
        "{text}"
    );
    assert!(
        text.contains("top       = 3        from config file"),
        "{text}"
    );

    // The environment beats the file; a flag beats both.
    let out = run(logsum(&dir)
        .env("LOGSUM_TOP", "7")
        .env("LOGSUM_COLOR", "auto")
        .args(["--show-config", "--color", "always"]));
    let text = stdout(&out);
    assert!(
        text.contains("min_level = error    from config file"),
        "{text}"
    );
    assert!(
        text.contains("top       = 7        from environment (LOGSUM_TOP)"),
        "{text}"
    );
    assert!(text.contains("color     = always   from flag"), "{text}");
}

#[test]
fn a_bad_config_file_explains_where_the_problem_is() {
    let dir = scratch_dir("bad_config");
    fs::write(dir.join("logsum.toml"), "top = \"three\"\n").unwrap();
    let out = run(logsum(&dir).arg(sample_log()));
    assert_eq!(out.status.code(), Some(2));
    let err = stderr(&out);
    assert!(err.contains("invalid config file"), "{err}");
    assert!(err.contains("line 1"), "{err}");
}

#[test]
fn colour_follows_the_flag_and_no_color() {
    let dir = scratch_dir("colour");
    let out = run(logsum(&dir).arg(sample_log()).args(["--color", "always"]));
    assert!(
        stdout(&out).contains("\x1b[31merror"),
        "always: colour even into a pipe"
    );

    let out = run(logsum(&dir).arg(sample_log()).env("NO_COLOR", "1"));
    assert!(!stdout(&out).contains('\x1b'));
}
