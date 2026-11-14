//! `cargo preflight` as a real custom Cargo command (Book 14.5).
//!
//! Cargo runs any program called `cargo-<name>` on your PATH when you type
//! `cargo <name>`. Install this one, then use it in any Cargo project:
//!
//!   cargo install --path 03_abstraction/05_cargo_and_crates_io --bin cargo-preflight
//!   cargo preflight
//!   cargo uninstall cargo_and_crates_io
//!
//! Or try it without installing:
//!   cargo run -p cargo_and_crates_io --bin cargo-preflight -- preflight

use std::process::ExitCode;

fn main() -> ExitCode {
    // `cargo preflight x` runs `cargo-preflight preflight x`: `run` skips the
    // extra `preflight` itself.
    cargo_and_crates_io::cli::run(std::env::args().skip(1))
}
