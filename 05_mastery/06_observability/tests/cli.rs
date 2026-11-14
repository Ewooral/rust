//! End-to-end: run the real program as a CHILD process, with `RUST_LOG`
//! set for that child only.
//!
//! Why a child process? `RUST_LOG` is read from the environment, and one
//! process has one environment, shared by every thread. Changing it inside
//! a test with `std::env::set_var` would change it under every other test
//! running in parallel; that's why `set_var` is an `unsafe fn` in edition
//! 2024. `Command::env` sets a variable for the child alone: safe, and no
//! test can disturb another.

use std::process::{Command, Output};

/// Runs the program; `rust_log: None` makes sure RUST_LOG is NOT set, even
/// if it happens to be set in the terminal running the tests.
fn run(rust_log: Option<&str>, args: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_observability"));
    command.args(args);
    match rust_log {
        Some(directives) => command.env("RUST_LOG", directives),
        None => command.env_remove("RUST_LOG"),
    };
    let output = command.output().expect("the program should start");
    assert!(output.status.success(), "{output:?}");
    output
}

/// Logs go to stderr.
fn log_lines(output: &Output) -> Vec<String> {
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .map(str::to_string)
        .collect()
}

#[test]
fn without_rust_log_the_level_is_info() {
    let output = run(None, &[]);
    let logs = log_lines(&output);
    assert!(logs.iter().any(|l| l.contains(" INFO ")));
    assert!(!logs.iter().any(|l| l.contains("DEBUG")), "{logs:#?}");
    // The summary is program OUTPUT, so it goes to stdout, not into the logs.
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("succeeded 4, failed 2"), "{stdout}");
}

#[test]
fn rust_log_warn_keeps_only_warnings_and_errors() {
    let logs = log_lines(&run(Some("warn"), &[]));
    assert_eq!(logs.len(), 3, "{logs:#?}");
    assert!(
        logs.iter()
            .all(|l| l.contains(" WARN ") || l.contains("ERROR"))
    );
}

#[test]
fn a_directive_can_turn_on_debug_for_one_module() {
    let logs = log_lines(&run(Some("warn,observability::processor=debug"), &[]));
    let debug = logs.iter().filter(|l| l.contains("DEBUG")).count();
    assert_eq!(debug, 7, "one `working` line per attempt: {logs:#?}");
    assert!(logs.iter().any(|l| l.contains("attempt{attempt=2}")));
}

#[test]
fn json_lines_are_valid_json_with_span_fields() {
    let logs = log_lines(&run(None, &["--json"]));
    let events: Vec<serde_json::Value> = logs
        .iter()
        .map(|line| serde_json::from_str(line).expect("every line is one JSON object"))
        .collect();
    let failed: Vec<&serde_json::Value> = events
        .iter()
        .filter(|e| e["fields"]["outcome"] == "failed")
        .collect();
    assert_eq!(failed.len(), 2);
    for event in failed {
        assert_eq!(event["level"], "ERROR");
        assert_eq!(event["span"]["name"], "job");
        assert!(event["span"]["job.id"].is_u64(), "{event}");
    }
}

#[test]
fn an_invalid_directive_is_reported_and_skipped() {
    let logs = log_lines(&run(Some("info,observability=loud"), &[]));
    assert!(
        logs[0].starts_with("ignoring `observability=loud`"),
        "{}",
        logs[0]
    );
    assert!(
        logs.iter().any(|l| l.contains("batch started")),
        "the rest still applies"
    );
}
