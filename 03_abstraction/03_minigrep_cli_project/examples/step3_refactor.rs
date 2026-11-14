//! Step 3 (Book 12.3): refactoring for modularity and error handling.
//! From 03_abstraction/03_minigrep_cli_project:
//!   cargo run --example step3_refactor -- the poem.txt
//!   cargo run --example step3_refactor                 (a friendly error now)
//!   cargo run --example step3_refactor -- the missing.txt
//!
//! The Book's guidelines for binary projects:
//!   - split the program into main.rs and lib.rs, logic in lib.rs;
//!   - main should only: parse arguments, set up configuration, call run,
//!     and handle the error run returns.
//!
//! This single file shows the shape; the real crate (src/) does the split.

use std::env;
use std::error::Error;
use std::fs;
use std::process;

// The values belong together, so group them: a struct, not a tuple of
// loose strings. (The Book's first version used `fn parse_config(..) ->
// (&str, &str)`; a struct says what each value MEANS.)
struct Config {
    query: String,
    file_path: String,
}

impl Config {
    // `build`, not `new`: by convention `new` never fails, and this can.
    fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }
        // clone: Config owns its Strings, so it doesn't borrow from args.
        // Simple, and the cost is tiny. Chapter 13 avoids it with iterators.
        let query = args[1].clone();
        let file_path = args[2].clone();
        Ok(Config { query, file_path })
    }
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?; // no more expect
    println!(
        "Searching for {:?}: the file has {} lines",
        config.query,
        contents.lines().count()
    );
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // unwrap_or_else: on Ok take the value; on Err run the closure, which
    // prints a message and exits with a NON-ZERO code (no panic, no
    // "thread 'main' panicked" noise for the user).
    let config = Config::build(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    // run returns () on success, so `if let Err` is all we need.
    if let Err(e) = run(config) {
        println!("Application error: {e}");
        process::exit(1);
    }
    // These errors still go to stdout with println!. Step 4 (in the real
    // crate's main.rs) moves them to stderr with eprintln!.
}
