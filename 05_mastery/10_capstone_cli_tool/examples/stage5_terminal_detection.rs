//! Stage 5: is a person watching? `std::io::IsTerminal` says whether stdin,
//! stdout or stderr is connected to a terminal, or to a file or pipe.
//! Run it three ways and compare:
//!   cargo run -p capstone_cli_tool --example stage5_terminal_detection
//!   cargo run -p capstone_cli_tool --example stage5_terminal_detection | cat
//!   echo hi | cargo run -p capstone_cli_tool --example stage5_terminal_detection 2> err.txt
//!   (then: cat err.txt, and delete it)

use std::env;
use std::io::{self, IsTerminal};

use capstone_cli_tool::{ColorWhen, Palette, use_color};

fn main() {
    let stdin = io::stdin().is_terminal();
    let stdout = io::stdout().is_terminal();
    let stderr = io::stderr().is_terminal();

    // Report on stderr too, so you can still see it when stdout is piped.
    eprintln!("stderr: stdin is a terminal:  {stdin}");
    eprintln!("stderr: stdout is a terminal: {stdout}");
    eprintln!("stderr: stderr is a terminal: {stderr}");

    let no_color = env::var("NO_COLOR").ok();
    let term = env::var("TERM").ok();
    println!("NO_COLOR = {no_color:?}, TERM = {term:?}");

    for when in [ColorWhen::Auto, ColorWhen::Always, ColorWhen::Never] {
        let colour = use_color(when, stdout, no_color.as_deref(), term.as_deref());
        let palette = Palette::new(colour);
        println!(
            "--color {when:<6} -> colour {:<5} {} {} {}",
            colour,
            palette.level(capstone_cli_tool::Level::Error, "error"),
            palette.level(capstone_cli_tool::Level::Warn, "warn"),
            palette.level(capstone_cli_tool::Level::Info, "info"),
        );
    }

    println!();
    println!("What a good CLI does with this:");
    println!("- stdout not a terminal  -> no colours (a file or `grep` would get escape codes)");
    println!("- stderr not a terminal  -> no progress line (a CI log would fill with \\r junk)");
    println!("- stdin IS a terminal    -> nobody piped input: print a hint, or an error");
}
