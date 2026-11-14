//! Solution to exercise 1: make it compile.
//!
//! `guess` is a `String` and `secret_number` is a `u32`, and Rust never
//! compares different types. Shadowing replaces the text with a number of
//! the same name, so the `match` is unchanged.
//!
//! Run: cargo run -p guessing_game --features solutions --bin sol01_fix_compare_string

use std::cmp::Ordering;

fn main() {
    let secret_number: u32 = 42;
    // Pretend the player typed 50 and pressed Enter.
    let guess = String::from("50\n");
    let guess: u32 = guess.trim().parse().expect("Please type a number!");

    match guess.cmp(&secret_number) {
        Ordering::Less => println!("Too small!"),
        Ordering::Greater => println!("Too big!"),
        Ordering::Equal => println!("You win!"),
    }
}
