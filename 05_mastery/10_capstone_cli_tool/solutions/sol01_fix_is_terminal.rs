//! Solution to exercise 1: make it compile (E0599).
//!
//! This program decides, the way logsum does, whether to colour its output
//! and whether to draw a progress line. It asks each stream
//! `is_terminal()`, but the compiler says there is no such method.
//!
//! The fix is one line: `use std::io::IsTerminal;`. `is_terminal` is a
//! method of the `IsTerminal` trait (implemented for Stdin, Stdout, Stderr,
//! File and more), and trait methods are only callable while the trait is in
//! scope.
//! Answer: colour decorates the RESULTS, which go to stdout, so ask stdout;
//! the progress line is drawn on stderr, so ask stderr. With `| cat`, stdout
//! is a pipe (no colour) while stderr is still your terminal (progress OK).
//!
//! Run: cargo run -p capstone_cli_tool --features solutions --bin sol01_fix_is_terminal
//!      cargo run -p capstone_cli_tool --features solutions --bin sol01_fix_is_terminal | cat

use std::env;
use std::io::{self, IsTerminal};

fn main() {
    // NO_COLOR set to anything non-empty turns colour off (https://no-color.org).
    let no_color = env::var("NO_COLOR").is_ok_and(|value| !value.is_empty());

    let colour = io::stdout().is_terminal() && !no_color;
    let progress = io::stderr().is_terminal();

    println!("colour the results:    {colour}");
    println!("draw a progress line:  {progress}");
}
