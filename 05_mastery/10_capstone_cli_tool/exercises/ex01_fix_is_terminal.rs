//! Exercise 1: make it compile (E0599).
//!
//! This program decides, the way logsum does, whether to colour its output
//! and whether to draw a progress line. It asks each stream
//! `is_terminal()`, but the compiler says there is no such method.
//!
//! The method comes from a TRAIT, `std::io::IsTerminal`, and a trait's
//! methods can only be called while the trait is in scope. Read the
//! compiler's `help:` lines, fix it with one `use`, then run it twice, once
//! normally and once piped, and compare the answers. Why does colour depend
//! on STDOUT but progress on STDERR?
//!
//! Run: cargo run -p capstone_cli_tool --features exercises --bin ex01_fix_is_terminal
//!      cargo run -p capstone_cli_tool --features exercises --bin ex01_fix_is_terminal | cat

use std::env;
use std::io;

fn main() {
    // NO_COLOR set to anything non-empty turns colour off (https://no-color.org).
    let no_color = env::var("NO_COLOR").is_ok_and(|value| !value.is_empty());

    let colour = io::stdout().is_terminal() && !no_color;
    let progress = io::stderr().is_terminal();

    println!("colour the results:    {colour}");
    println!("draw a progress line:  {progress}");
}
