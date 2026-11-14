//! Where does Cargo put the program it builds?
//! Run: cargo run -p hello_world --example where_is_my_program
//! Then: cargo run -p hello_world --example where_is_my_program --release
//!
//! Compare the two paths: `target/debug/...` and `target/release/...`.

fn main() {
    // `current_exe` asks the operating system where this running program is.
    let me = std::env::current_exe().expect("the OS should know where we are");
    println!("I am running from:\n  {}", me.display());

    // `env!` reads values Cargo hands the compiler while it builds us.
    println!("package: {}", env!("CARGO_PKG_NAME"));
    println!("version: {}", env!("CARGO_PKG_VERSION"));
    println!("manifest folder: {}", env!("CARGO_MANIFEST_DIR"));

    // `debug_assertions` is on for `cargo build`/`cargo run`, off for `--release`.
    if cfg!(debug_assertions) {
        println!("profile: dev (unoptimised, extra checks, quick to compile)");
    } else {
        println!("profile: release (optimised: slower to compile, faster to run)");
    }
}
