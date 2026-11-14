//! Mini project for Book chapter 14: `cargo preflight`, a checker that tells
//! you whether a crate's Cargo.toml is ready for crates.io.
//!
//!   cargo run -p cargo_and_crates_io                           whole workspace
//!   cargo run -p cargo_and_crates_io -- 03_abstraction/05_cargo_and_crates_io
//!   cargo run -p cargo_and_crates_io --no-default-features     plain ASCII marks
//!   cargo run -p cargo_and_crates_io --features json -- 03_abstraction/05_cargo_and_crates_io --json
//!   cargo test -p cargo_and_crates_io
//!
//! All the logic lives in the library (src/lib.rs). The same program is also
//! built as `cargo-preflight` (src/bin/cargo-preflight.rs), the name Cargo
//! looks for when you type `cargo preflight`.

use std::process::ExitCode;

fn main() -> ExitCode {
    cargo_and_crates_io::cli::run(std::env::args().skip(1))
}
