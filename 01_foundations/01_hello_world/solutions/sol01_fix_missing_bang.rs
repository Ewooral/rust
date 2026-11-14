//! Solution to exercise 1: make it compile.
//!
//! `println` is a macro, and macros are called with `!`. Without the `!`,
//! Rust looks for a *function* called `println`, and there isn't one.
//!
//! Run: cargo run -p hello_world --features solutions --bin sol01_fix_missing_bang

fn main() {
    println!("Hello, world!");
}
