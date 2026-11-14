//! Workspace helper commands (the "xtask" pattern), run through the alias in
//! `.cargo/config.toml`:
//!
//!   cargo xtask fmt            format every package
//!   cargo xtask fmt --check    check formatting without changing files
//!   cargo xtask ci             fmt --check, then clippy, then tests, like CI would
//!
//! Why not just `cargo fmt --all`? It hands every Rust file in the workspace
//! to rustfmt in ONE command, and on Windows that command line is now longer
//! than the operating system allows ("The filename or extension is too long",
//! os error 206). Formatting one package at a time stays under the limit and
//! works the same on Linux.
//!
//! An xtask is an ordinary binary crate in the workspace: plain Rust instead
//! of shell scripts, so it runs identically on every platform.

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("fmt") => fmt(args.iter().any(|a| a == "--check")),
        Some("ci") => ci(),
        _ => {
            eprintln!("usage: cargo xtask <fmt [--check] | ci>");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("xtask: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The `cargo` that launched us (Cargo sets CARGO), or `cargo` from PATH.
fn cargo() -> Command {
    Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
}

/// Names of every package in the workspace, from `cargo metadata`.
fn packages() -> Result<Vec<String>, String> {
    let output = cargo()
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .map_err(|e| format!("could not run cargo metadata: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("cargo metadata gave invalid JSON: {e}"))?;
    let mut names: Vec<String> = json["packages"]
        .as_array()
        .ok_or("cargo metadata has no packages list")?
        .iter()
        .filter_map(|p| p["name"].as_str().map(String::from))
        .collect();
    names.sort();
    Ok(names)
}

fn fmt(check: bool) -> Result<(), String> {
    let packages = packages()?;
    let mut failed = Vec::new();
    for name in &packages {
        let mut cmd = cargo();
        cmd.args(["fmt", "-p", name]);
        if check {
            cmd.args(["--", "--check"]);
        }
        let ok = cmd
            .status()
            .map_err(|e| format!("could not run cargo fmt: {e}"))?
            .success();
        if !ok {
            failed.push(name.as_str());
        }
    }
    if failed.is_empty() {
        let verb = if check { "checked" } else { "formatted" };
        println!("{verb} {} packages", packages.len());
        Ok(())
    } else if check {
        Err(format!(
            "not formatted: {} (run `cargo xtask fmt`)",
            failed.join(", ")
        ))
    } else {
        Err(format!("rustfmt failed for: {}", failed.join(", ")))
    }
}

fn ci() -> Result<(), String> {
    fmt(true)?;
    let steps: [&[&str]; 2] = [
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
        &["test", "--workspace"],
    ];
    for step in steps {
        println!("cargo {}", step.join(" "));
        let ok = cargo()
            .args(step)
            .status()
            .map_err(|e| format!("could not run cargo: {e}"))?
            .success();
        if !ok {
            return Err(format!("`cargo {}` failed", step.join(" ")));
        }
    }
    println!("all checks passed");
    Ok(())
}
