//! Book 14.5: extending Cargo with a custom command.
//!
//! Run it the way Cargo would:
//!   cargo run -p cargo_and_crates_io --example cargo-greet -- greet Ama
//!
//! Or make `cargo greet` a real command. Cargo runs any program called
//! `cargo-<name>` on your PATH when you type `cargo <name>`:
//!   cargo build -p cargo_and_crates_io --example cargo-greet
//!   bash:        PATH="$PATH:$PWD/target/debug/examples" cargo greet Ama
//!   PowerShell:  $env:PATH += ";$PWD\target\debug\examples"; cargo greet Ama
//! (Run those from the repository root. If you set CARGO_TARGET_DIR, the
//! program is in $CARGO_TARGET_DIR/debug/examples instead.)

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("started as: {args:?}");

    // `cargo greet Ama` runs `cargo-greet greet Ama`: Cargo passes the
    // command's own name first, so skip it if it is there.
    let mut rest = args.iter().skip(1).map(String::as_str).peekable();
    rest.next_if_eq(&"greet");
    let names: Vec<&str> = rest.collect();

    let who = if names.is_empty() {
        "Rustacean".to_string()
    } else {
        names.join(" and ")
    };
    println!("Hello, {who}!");

    // Cargo tells the command where Cargo itself is, so it can run
    // `cargo build` or `cargo metadata` for you.
    match env::var("CARGO") {
        Ok(path) => println!("CARGO = {path}"),
        Err(_) => println!("CARGO is not set: Cargo did not start me"),
    }
}
