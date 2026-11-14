//! Colours in the terminal with the `colored` crate (version 3).
//! Run: cargo run -p guessing_game --example colored_output
//!
//! Colours only appear when printing to a real terminal. Pipe the output
//! (`... | cat`) or set `NO_COLOR=1` and the same program prints plain text.

// `Colorize` is a trait. Importing it adds methods like `.red()` to every
// `&str` and `String`, a pattern called an "extension trait".
use colored::Colorize;

fn main() {
    println!("{}", "Too small!".red());
    println!("{}", "Too big!".red());
    println!("{}", "You win!".green().bold());
    println!("{}", "Enter a number from 1 to 100.".yellow());
    println!("{}", "white on blue".white().on_blue());

    // The methods return a `ColoredString`, which prints with colour codes
    // around the text. `format!` first when the text has placeholders.
    let attempts = 4;
    println!("{}", format!("You win in {attempts} attempt(s)!").green());
}
