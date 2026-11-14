//! The command line program, shared by both binaries:
//! `cargo_and_crates_io` (src/main.rs) and `cargo-preflight`
//! (src/bin/cargo-preflight.rs, the custom Cargo command of Book 14.5).

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::checks::{Finding, check, passes};
use crate::manifest::Manifest;
use crate::report;
use crate::workspace::{find_manifest, find_workspace_root, member_dirs};

const USAGE: &str = "\
usage: cargo preflight [PATH] [--json]

PATH      a Cargo.toml, or a folder inside a Cargo project (default: here)
--json    print JSON (needs the `json` feature)

A workspace root: one summary line per member (always exit code 0).
A single crate: every finding, and exit code 1 if crates.io would refuse it.";

/// Runs the program with the arguments that follow the program's name.
///
/// When Cargo runs a custom command, `cargo preflight x` starts the program
/// `cargo-preflight` with the arguments `preflight x`. So a first argument
/// of `preflight` is skipped.
pub fn run(args: impl IntoIterator<Item = String>) -> ExitCode {
    let mut args: Vec<String> = args.into_iter().collect();
    if args.first().is_some_and(|a| a == "preflight") {
        args.remove(0);
    }

    let mut json = false;
    let mut path = None;
    for arg in args {
        match arg.as_str() {
            "--json" => json = true,
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            flag if flag.starts_with('-') => {
                eprintln!("unknown option `{flag}`\n\n{USAGE}");
                return ExitCode::from(2);
            }
            _ => path = Some(PathBuf::from(arg)),
        }
    }
    if json && !cfg!(feature = "json") {
        eprintln!(
            "--json needs the `json` feature:\n  \
             cargo run -p cargo_and_crates_io --features json -- --json"
        );
        return ExitCode::from(2);
    }

    let start = path.unwrap_or_else(|| env::current_dir().unwrap_or_default());
    let Some(manifest_path) = find_manifest(&start) else {
        eprintln!(
            "no Cargo.toml in {} or any folder above it",
            start.display()
        );
        return ExitCode::FAILURE;
    };
    match load(&manifest_path) {
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
        Ok(manifest) if manifest.is_workspace() && !manifest.has_section("package") => {
            summarise_workspace(&manifest_path, &manifest);
            ExitCode::SUCCESS
        }
        Ok(mut manifest) => {
            if let Some((_, root)) = find_workspace_root(&manifest_path) {
                manifest.inherit_from(&root);
            }
            let findings = check(&manifest);
            print_one(&manifest_path, &manifest, &findings, json);
            if passes(&findings) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
    }
}

fn load(path: &Path) -> Result<Manifest, String> {
    let text =
        fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    Manifest::parse(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn summarise_workspace(root_path: &Path, root: &Manifest) {
    let root_dir = root_path.parent().unwrap_or(Path::new("."));
    let members = member_dirs(root_dir, root);
    println!(
        "Preflight for the workspace at {} ({} members)\n",
        root_dir.display(),
        members.len()
    );
    let mut ready = 0;
    for dir in &members {
        let fallback = dir.display().to_string();
        match load(&dir.join("Cargo.toml")) {
            Err(message) => println!("  ?    {fallback}: {message}"),
            Ok(mut manifest) => {
                manifest.inherit_from(root);
                let findings = check(&manifest);
                let name = manifest.package_str("name").unwrap_or(&fallback);
                println!("{}", report::summary_line(name, &findings));
                ready += usize::from(passes(&findings));
            }
        }
    }
    println!(
        "\n{ready} of {} crates are ready for crates.io.",
        members.len()
    );
    println!("Give a crate's folder for the details: cargo preflight <folder>");
}

fn print_one(path: &Path, manifest: &Manifest, findings: &[Finding], json: bool) {
    let name = manifest.package_str("name").unwrap_or("(no name)");
    if json {
        #[cfg(feature = "json")]
        match report::render_json(name, findings) {
            Ok(text) => println!("{text}"),
            Err(e) => eprintln!("could not write JSON: {e}"),
        }
        return;
    }
    let version = manifest.package_str("version").unwrap_or("?");
    let title = format!("{name} {version} ({})", path.display());
    print!("{}", report::render(&title, findings));
}
