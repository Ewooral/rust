//! The build script for this lesson (Beyond the Book).
//!
//! Cargo compiles and runs this file BEFORE it compiles the crate itself.
//! It talks back to Cargo by printing `cargo::` lines on standard output.
//! This one does two jobs:
//!
//! 1. Code generation: it turns `licenses.txt` (plain data) into Rust code at
//!    `$OUT_DIR/licenses.rs`, which `src/licenses.rs` pulls in with `include!`.
//! 2. It passes facts that only build scripts are told (the profile, the
//!    opt-level, the target) on to the crate as `BUILD_*` environment
//!    variables, read with `env!` in `examples/release_profiles.rs`.

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Without these lines Cargo re-runs the script whenever ANY file in the
    // package changes. With them, only when these two files change.
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=licenses.txt");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("set by Cargo"));
    let list = fs::read_to_string(manifest_dir.join("licenses.txt"))
        .expect("licenses.txt should sit next to Cargo.toml");

    let mut ids: Vec<&str> = list
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    ids.sort_unstable(); // sorted, so the crate can binary_search it
    ids.dedup();

    let mut code = String::from(
        "/// SPDX licence identifiers this checker knows, sorted.\n\
         ///\n\
         /// Generated at build time by `build.rs` from `licenses.txt`.\n\
         pub const KNOWN_LICENSES: &[&str] = &[\n",
    );
    for id in &ids {
        writeln!(code, "    {id:?},").expect("writing to a String cannot fail");
    }
    code.push_str("];\n");

    // OUT_DIR is a private folder for generated files, inside target/.
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("set by Cargo"));
    fs::write(out_dir.join("licenses.rs"), code).expect("OUT_DIR should be writable");

    // Cargo tells build scripts about the build in environment variables.
    // `cargo::rustc-env` passes them on to every target of this package.
    for var in ["PROFILE", "OPT_LEVEL", "DEBUG", "TARGET"] {
        let value = env::var(var).unwrap_or_else(|_| String::from("unknown"));
        println!("cargo::rustc-env=BUILD_{var}={value}");
    }
}
