//! Exercise 2: make it compile (E0277), with `thiserror`'s `#[from]`.
//!
//! `load_top` reads a file holding one number. It uses `?` on an
//! `io::Error` and on a `ParseIntError`, but returns `Result<_, ConfigError>`.
//! `?` converts errors with `From`, and nobody has said how to turn those two
//! into a `ConfigError`, so the compiler refuses (twice).
//!
//! 1. Add `#[from]` to the field of each variant. thiserror then writes the
//!    two `From` impls for you, and also makes each field the error's
//!    `source()`. Run it and read the printed chain.
//! 2. Think: the message for a missing file doesn't say WHICH file. `#[from]`
//!    can't know the path. How does logsum's `ConfigError::Read` (in
//!    src/config.rs) solve that? (Answer in the solution.)
//!
//! Run: cargo run -p capstone_cli_tool --features exercises --bin ex02_fix_error_conversion

use std::error::Error;
use std::fs;
use std::io;
use std::num::ParseIntError;
use std::path::Path;

use thiserror::Error;

#[derive(Debug, Error)]
enum ConfigError {
    #[error("cannot read the config file")]
    Io(io::Error),
    #[error("the config file must hold a whole number")]
    BadNumber(ParseIntError),
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
