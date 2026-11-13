//! Beyond the Book: the two error crates almost every Rust project uses.
//! Run: cargo run -p error_handling --example thiserror_and_anyhow
//!
//! - `thiserror` (for LIBRARIES): derive Display, Error and From for your own
//!   error enum, instead of writing them by hand. Callers can still `match`
//!   on each variant.
//! - `anyhow` (for APPLICATIONS): one error type, `anyhow::Error`, that holds
//!   any error, plus `.context(...)` to say what you were doing when it failed.

use std::num::ParseIntError;

// ---------------- library side: thiserror ----------------

#[derive(Debug, thiserror::Error)]
enum ConfigError {
    #[error("missing key `{0}`")]
    MissingKey(&'static str),

    // #[from] writes `impl From<ParseIntError> for ConfigError`, so `?`
    // converts automatically; it also becomes this error's source().
    #[error("not a whole number")]
    BadNumber(#[from] ParseIntError),

    #[error("{value} is out of range {min}..={max}")]
    OutOfRange { value: u32, min: u32, max: u32 },
}

fn read_threads(text: &str) -> Result<u32, ConfigError> {
    let raw = text
        .strip_prefix("threads=")
        .ok_or(ConfigError::MissingKey("threads"))?;
    let value: u32 = raw.trim().parse()?; // ParseIntError -> ConfigError
    if !(1..=64).contains(&value) {
        return Err(ConfigError::OutOfRange {
            value,
            min: 1,
            max: 64,
        });
    }
    Ok(value)
}

// ---------------- application side: anyhow ----------------

use anyhow::{Context, Result, bail};

/// `anyhow::Result<T>` is `Result<T, anyhow::Error>`.
fn load(name: &str, text: &str) -> Result<u32> {
    if name.is_empty() {
        bail!("a config needs a name"); // make an error from a message
    }
    let threads = read_threads(text).with_context(|| format!("while loading config `{name}`"))?;
    Ok(threads)
}

fn main() {
    for (name, text) in [
        ("web", "threads=8"),
        ("db", "threads=x"),
        ("cache", "colour=blue"),
        ("", ""),
    ] {
        match load(name, text) {
            Ok(n) => println!("{name}: {n} threads"),
            // {:#} prints the whole chain: context, then each cause.
            Err(e) => println!("error: {e:#}"),
        }
    }

    // A library error can still be matched precisely.
    if let Err(ConfigError::OutOfRange { value, .. }) = read_threads("threads=100") {
        println!("matched OutOfRange with value {value}");
    }
}
