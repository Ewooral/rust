//! Stage 6: a progress line and colours, made by hand with `\r` and ANSI
//! escape codes, so you can see there's no magic in crates like `indicatif`.
//! Run: cargo run -p capstone_cli_tool --example stage6_progress_and_colors
//! Then: cargo run -p capstone_cli_tool --example stage6_progress_and_colors 2> NUL    (PowerShell/cmd)
//!       cargo run -p capstone_cli_tool --example stage6_progress_and_colors 2> /dev/null (bash)
//! The progress line is on stderr, so hiding stderr hides only the progress.

use std::io::{self, IsTerminal};
use std::thread;
use std::time::Duration;

use capstone_cli_tool::{Level, Palette, Progress};

fn main() {
    // Only draw progress when a person is watching stderr.
    let show = io::stderr().is_terminal();
    let total: u64 = 3 * 1024 * 1024; // pretend to read a 3 MiB file
    let mut progress = Progress::new(io::stderr(), show, "pretend.log", Some(total));

    let mut done = 0;
    while done < total {
        thread::sleep(Duration::from_millis(15)); // "work"
        done = (done + 40 * 1024).min(total);
        progress.update(done); // redraws at most every 100 ms
    }
    progress.finish(); // clear the line before the real output

    // The palette from the library: escape codes only if stdout is a terminal.
    let palette = Palette::new(io::stdout().is_terminal());
    println!("done: {} bytes", done);
    for level in Level::ALL.iter().rev() {
        // Pad first, then paint (padding a painted string counts the
        // invisible escape bytes and breaks the alignment).
        println!(
            "{} | {}",
            palette.level(*level, &format!("{level:<5}")),
            level.index()
        );
    }
    println!(
        "{} {}",
        palette.bold("bold"),
        palette.dim("dim (if your terminal supports it)")
    );
    if !palette.enabled() {
        println!("(stdout is not a terminal, so no colours)");
    }
}
