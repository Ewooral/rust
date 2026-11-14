//! Solution to exercise 2: make it compile (E0277), with `thiserror`'s `#[from]`.
//!
//! `load_top` reads a file holding one number. It uses `?` on an
//! `io::Error` and on a `ParseIntError`, but returns `Result<_, ConfigError>`.
//! `?` converts errors with `From`, and nobody had said how to turn those two
//! into a `ConfigError`.
//!
//! `#[from]` on each field makes thiserror write
//! `impl From<io::Error> for ConfigError` (and the same for ParseIntError),
//! and marks the field as the `source()`.
//!
//! Answer to the question: `#[from]` only receives the io::Error, so it
//! can't know which file failed. logsum's `ConfigError::Read` has a `path`
//! field and a `#[source]` (not `#[from]`), and builds the error by hand
//! with `.map_err(|source| ConfigError::Read { path: path.to_path_buf(), source })`.
//! Use `#[from]` when the inner error says everything; build the variant
//! yourself when you need to add facts such as a path.
//!
//! Run: cargo run -p capstone_cli_tool --features solutions --bin sol02_fix_error_conversion

use std::error::Error;
use std::fs;
use std::io;
use std::num::ParseIntError;
use std::path::Path;

use thiserror::Error;

#[derive(Debug, Error)]
enum ConfigError {
    #[error("cannot read the config file")]
    Io(#[from] io::Error),
    #[error("the config file must hold a whole number")]
    BadNumber(#[from] ParseIntError),
}

fn load_top(path: &Path) -> Result<usize, ConfigError> {
    let text = fs::read_to_string(path)?;
    let top = text.trim().parse::<usize>()?;
    Ok(top)
}

fn main() {
    let path = std::env::temp_dir().join("logsum_ex02_top.txt");
    for contents in [Some("7\n"), Some("seven\n"), None] {
        match contents {
            Some(text) => fs::write(&path, text).expect("can write to the temp folder"),
            None => {
                let _ = fs::remove_file(&path);
            }
        }
        match load_top(&path) {
            Ok(top) => println!("top = {top}"),
            Err(e) => {
                print!("error: {e}");
                if let Some(cause) = e.source() {
                    print!("\n  caused by: {cause}");
                }
                println!();
            }
        }
    }
}
