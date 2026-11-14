//! Solution to exercise 3: make it compile.
//!
//! The annotation `: u32` tells `parse` which type to produce. The
//! "turbofish" `"42".parse::<u32>()` would work too.
//!
//! Run: cargo run -p variables_and_data_types --features solutions --bin sol03_fix_parse_annotation

fn main() {
    let guess: u32 = "42".parse().expect("Not a number!");
    println!("You guessed {guess}");
}
