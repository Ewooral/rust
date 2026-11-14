//! The mini project: load, upgrade and check backup settings files.
//!
//! ```text
//! cargo run -p serialization                                   # every file in samples/
//! cargo run -p serialization -- 05_mastery/07_serialization/samples/1_v1_family_photos.json
//! cargo run -p serialization -- <file> --json                  # write the result as JSON
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serialization::{Format, LoadError, MigrateError, load};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let output = if args.iter().any(|a| a == "--json") {
        Format::Json
    } else {
        Format::Toml
    };
    match args.iter().find(|a| !a.starts_with("--")) {
        Some(path) => show_one(Path::new(path), output),
        None => {
            show_samples();
            ExitCode::SUCCESS
        }
    }
}

/// One line per sample file: how it went.
fn show_samples() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("the samples folder is part of the lesson")
        .map(|entry| entry.expect("readable folder entry").path())
        .collect();
    files.sort();
    println!("Loading every file in {}\n", dir.display());
    for path in files {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let Some(format) = Format::from_path(&path) else {
            continue;
        };
        let text = std::fs::read_to_string(&path).expect("readable sample");
        match load(&text, format) {
            Ok(migrated) if migrated.from_version == migrated.settings.version => {
                println!("OK    {name:<26} version {}", migrated.from_version);
            }
            Ok(migrated) => println!(
                "OK    {name:<26} upgraded v{} -> v{} ({} changes)",
                migrated.from_version,
                migrated.settings.version,
                migrated.notes.len()
            ),
            Err(err) => {
                let message = err.to_string();
                let mut lines = message.lines();
                println!("FAIL  {name:<26} {}", lines.next().unwrap_or_default());
                for line in lines {
                    println!("      {line}");
                }
                if let LoadError::Migrate(MigrateError::Parse(parse)) = &err {
                    for line in parse.snippet(&text).lines() {
                        println!("      {line}");
                    }
                }
            }
        }
    }
    println!("\nPass one file to see its upgrade in full:");
    println!(
        "  cargo run -p serialization -- 05_mastery/07_serialization/samples/1_v1_family_photos.json"
    );
}

/// Load one file, explain what changed, and print it in the current format.
fn show_one(path: &Path, output: Format) -> ExitCode {
    let Some(format) = Format::from_path(path) else {
        eprintln!("{}: expected a .json or .toml file", path.display());
        return ExitCode::FAILURE;
    };
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("{}: {err}", path.display());
            return ExitCode::FAILURE;
        }
    };
    match load(&text, format) {
        Ok(migrated) => {
            println!(
                "{}: {format}, version {}",
                path.display(),
                migrated.from_version
            );
            for note in &migrated.notes {
                println!("  - {note}");
            }
            let written = output
                .write(&migrated.settings)
                .expect("Settings can always be written");
            println!("\nAs the current version, in {output}:\n\n{written}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{}: {err}", path.display());
            if let LoadError::Migrate(MigrateError::Parse(parse)) = &err {
                eprintln!("{}", parse.snippet(&text));
            }
            ExitCode::FAILURE
        }
    }
}
