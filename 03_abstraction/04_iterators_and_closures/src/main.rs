//! minigrep v2 (Book 13.3). From this folder:
//!   cargo run -- to poem.txt
//!   cargo run -- --summary poem.txt     (Beyond the Book: a word report)
//!
//! The only change from chapter 12's main: we pass `env::args()` itself to
//! Config::build instead of collecting it into a Vec first.

use std::env;
use std::fs;
use std::process;

use iterators_and_closures::{Config, run, summary};

fn main() {
    // Extra mode: `--summary <file>` prints a report instead of searching.
    let mut peek = env::args().skip(1);
    if peek.next().as_deref() == Some("--summary") {
        let path = peek.next().unwrap_or_else(|| {
            eprintln!("Usage: --summary <file_path>");
            process::exit(1);
        });
        match fs::read_to_string(&path) {
            Ok(text) => {
                let (lines, words, top) = summary(&text);
                println!("{path}: {lines} lines, {words} words, most frequent long word: {top:?}");
            }
            Err(e) => {
                eprintln!("Application error: {e}");
                process::exit(1);
            }
        }
        return;
    }

    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
