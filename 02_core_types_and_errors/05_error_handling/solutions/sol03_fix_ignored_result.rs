//! Solution to exercise 3: make it compile (unused_must_use).
//!
//! Ignoring a Result is normally a WARNING. This file turns that warning into
//! an error with `#![deny(...)]`, the way many teams do in CI.
//! Handle the Result: print the error if saving failed. Do not use
//! `let _ =` (that would hide the failure) and do not delete the deny line.
//!
//! Run: cargo run -p error_handling --features solutions --bin sol03_fix_ignored_result

#![deny(unused_must_use)]

fn save_score(name: &str, score: u32) -> Result<(), String> {
    if score > 100 {
        return Err(format!("{score} is not a valid score for {name}"));
    }
    println!("saved {name}: {score}");
    Ok(())
}

fn main() {
    for (name, score) in [("Ama", 90), ("Kofi", 120)] {
        if let Err(e) = save_score(name, score) {
            println!("could not save: {e}");
        }
    }
}
