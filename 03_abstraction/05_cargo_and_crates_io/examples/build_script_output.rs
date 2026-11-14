//! Beyond the Book: what the build script (build.rs) made for this crate.
//!
//! Run: cargo run -p cargo_and_crates_io --example build_script_output
//!
//! Before compiling the crate, Cargo compiles and runs build.rs. It read
//! licenses.txt and wrote Rust code into OUT_DIR, and it passed the profile
//! and target on as BUILD_* environment variables.

use std::fs;
use std::path::Path;

use cargo_and_crates_io::KNOWN_LICENSES;

fn main() -> std::io::Result<()> {
    // OUT_DIR is a private folder inside target/ for generated files.
    let out_dir = Path::new(env!("OUT_DIR"));
    println!("OUT_DIR: {}\n", out_dir.display());

    // The generated file is ordinary Rust. Here are its first lines:
    let generated = fs::read_to_string(out_dir.join("licenses.rs"))?;
    for line in generated.lines().take(6) {
        println!("  | {line}");
    }
    println!("  | ...\n");

    // ...and src/licenses.rs pulls it in with include!, so the crate has a
    // constant it never typed:
    println!("KNOWN_LICENSES has {} entries:", KNOWN_LICENSES.len());
    for row in KNOWN_LICENSES.chunks(6) {
        println!("  {}", row.join("  "));
    }

    // Facts that only build scripts are told, forwarded with cargo::rustc-env:
    println!("\nPROFILE={}", env!("BUILD_PROFILE"));
    println!("OPT_LEVEL={}", env!("BUILD_OPT_LEVEL"));
    println!("TARGET={}", env!("BUILD_TARGET"));
    Ok(())
}
