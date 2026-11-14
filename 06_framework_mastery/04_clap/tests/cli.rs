//! End-to-end tests: run the real `clap_cli` binary as a separate PROCESS,
//! the way a shell would, and check stdout, stderr and the exit code.
//!
//! Cargo builds the binary first and tells us where it is through the
//! `CARGO_BIN_EXE_<name>` environment variable. Every test gets its own
//! temporary folder for the task file, and `TASKS_STORE` is set or removed
//! on the CHILD process only, so the tests never depend on (or change) the
//! environment of the shell running them.
//!
//! Run: cargo test -p clap_cli --test cli

use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn finish(output: Output) -> Run {
    Run {
        code: output.status.code().expect("exited normally"),
        stdout: String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        stderr: String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n"),
    }
}

fn clap_cli() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_clap_cli"));
    cmd.env_remove("TASKS_STORE");
    cmd
}

/// Runs `clap_cli --store <dir>/tasks.json <args>`.
fn run(dir: &Path, args: &[&str]) -> Run {
    let store = dir.join("tasks.json");
    finish(
        clap_cli()
            .arg("--store")
            .arg(&store)
            .args(args)
            .output()
            .expect("clap_cli should start"),
    )
}

#[test]
fn help_and_version_exit_with_0() {
    let help = finish(clap_cli().arg("--help").output().unwrap());
    assert_eq!(help.code, 0);
    assert!(
        help.stdout.contains("Usage: clap_cli [OPTIONS] <COMMAND>"),
        "{}",
        help.stdout
    );
    assert!(help.stdout.contains("list         List tasks [alias: ls]"));

    let version = finish(clap_cli().arg("--version").output().unwrap());
    assert_eq!(
        (version.code, version.stdout.as_str()),
        (0, "clap_cli 0.1.0\n")
    );
}

#[test]
fn add_list_done_remove_round_trip() {
    let dir = TempDir::new().unwrap();
    let d = dir.path();
    assert_eq!(
        run(d, &["add", "Buy", "milk", "-t", "home"]).stdout,
        "added task 1: Buy milk\n"
    );
    assert_eq!(
        run(d, &["add", "Ship it", "-p", "high", "-d", "2026-12-01"]).stdout,
        "added task 2: Ship it\n"
    );

    let table = run(d, &["list"]);
    assert_eq!(table.code, 0);
    assert_eq!(
        table.stdout,
        " ID  PRI     DONE  DUE         TAGS            TITLE\n  \
           1  normal        -           home            Buy milk\n  \
           2  high          2026-12-01                  Ship it\n"
    );

    assert_eq!(run(d, &["done", "1"]).stdout, "done: 1\n");
    assert_eq!(run(d, &["ls", "-f", "plain"]).stdout, "2\tShip it\n");
    assert_eq!(run(d, &["rm", "--done"]).stdout, "removed 1 task(s)\n");
    assert_eq!(
        run(d, &["list", "--status", "all", "-f", "plain"]).stdout,
        "2\tShip it\n"
    );
}

#[test]
fn json_output_is_machine_readable() {
    let dir = TempDir::new().unwrap();
    run(dir.path(), &["add", "Write", "docs", "-t", "work"]);
    let out = run(dir.path(), &["list", "--format", "json"]);
    let tasks: serde_json::Value = serde_json::from_str(&out.stdout).unwrap();
    assert_eq!(tasks[0]["title"], "Write docs");
    assert_eq!(tasks[0]["priority"], "normal");
    assert_eq!(tasks[0]["tags"][0], "work");
}

#[test]
fn the_store_comes_from_the_flag_then_the_env_var_then_the_default() {
    let dir = TempDir::new().unwrap();
    let from_env = dir.path().join("env.json");
    let out = finish(
        clap_cli()
            .env("TASKS_STORE", &from_env) // only for this child process
            .args(["add", "from", "env"])
            .output()
            .unwrap(),
    );
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(from_env.exists(), "TASKS_STORE was used");

    let from_flag = dir.path().join("flag.json");
    finish(
        clap_cli()
            .env("TASKS_STORE", &from_env)
            .arg("--store")
            .arg(&from_flag)
            .args(["add", "from", "flag"])
            .output()
            .unwrap(),
    );
    assert!(from_flag.exists(), "--store beats TASKS_STORE");

    // With neither, the default `tasks.json` in the current folder.
    finish(
        clap_cli()
            .current_dir(dir.path())
            .args(["add", "x"])
            .output()
            .unwrap(),
    );
    assert!(dir.path().join("tasks.json").exists());
}

#[test]
fn bad_arguments_exit_with_2_and_explain_on_stderr() {
    let dir = TempDir::new().unwrap();
    let out = run(dir.path(), &["add", "x", "--priority", "urgent"]);
    assert_eq!(out.code, 2);
    assert!(out.stdout.is_empty());
    assert!(
        out.stderr
            .contains("invalid value 'urgent' for '--priority <PRIORITY>'"),
        "{}",
        out.stderr
    );
    assert!(out.stderr.contains("[possible values: low, normal, high]"));

    let out = run(dir.path(), &["add", "x", "--due", "2026-02-30"]);
    assert_eq!(out.code, 2);
    assert!(
        out.stderr.contains("2026-02 has no day 30"),
        "{}",
        out.stderr
    );

    assert_eq!(
        run(dir.path(), &["remove"]).code,
        2,
        "ids or --done is required"
    );
    assert_eq!(
        run(dir.path(), &["remove", "1", "--done"]).code,
        2,
        "not both"
    );
    assert_eq!(run(dir.path(), &["frobnicate"]).code, 2);
}

#[test]
fn failed_commands_exit_with_1() {
    let dir = TempDir::new().unwrap();
    let out = run(dir.path(), &["done", "42"]);
    assert_eq!(out.code, 1);
    assert_eq!(out.stderr, "error: no task with id 42\n");

    std::fs::write(dir.path().join("tasks.json"), "not json").unwrap();
    let out = run(dir.path(), &["list"]);
    assert_eq!(out.code, 1);
    assert!(
        out.stderr.contains("is not a valid task file"),
        "{}",
        out.stderr
    );
}

#[test]
fn verbose_notes_go_to_stderr() {
    let dir = TempDir::new().unwrap();
    let out = run(dir.path(), &["-vv", "add", "x"]);
    assert_eq!(out.stdout, "added task 1: x\n");
    assert!(out.stderr.contains("note: using"), "{}", out.stderr);
    assert!(out.stderr.contains("note: saved"), "{}", out.stderr);
}

#[test]
fn completions_are_printed_to_stdout() {
    let out = finish(clap_cli().args(["completions", "bash"]).output().unwrap());
    assert_eq!(out.code, 0);
    assert!(out.stdout.contains("_clap_cli()"), "a bash function");
    let out = finish(
        clap_cli()
            .args(["completions", "powershell"])
            .output()
            .unwrap(),
    );
    assert!(out.stdout.contains("Register-ArgumentCompleter"));
    assert_eq!(
        finish(clap_cli().args(["completions", "tcsh"]).output().unwrap()).code,
        2
    );
}
