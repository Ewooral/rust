//! Exercise 1: make it compile.
//!
//! This is the Book's famous mistake: comparing the text the player typed
//! with a number (E0308 mismatched types). Fix it the Book's way: shadow
//! `guess` with a `u32` made by trimming and parsing the text. Don't change
//! the `match`.
//!
//! Run: cargo run -p guessing_game --features exercises --bin ex01_fix_compare_string

use std::cmp::Ordering;

fn main() {
    let secret_number: u32 = 42;
    // Pretend the player typed 50 and pressed Enter.
    let guess = String::from("50\n");

    match guess.cmp(&secret_number) {
        Ordering::Less => println!("Too small!"),
        Ordering::Greater => println!("Too big!"),
        Ordering::Equal => println!("You win!"),
    }
}
