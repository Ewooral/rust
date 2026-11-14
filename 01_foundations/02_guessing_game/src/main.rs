//! Mini project for Book chapter 2: the guessing game, finished.
//!
//! This is the Book's program plus a few upgrades: the range lives in
//! constants, out-of-range guesses are rejected with a match guard, attempts
//! are counted, hints are coloured, and the game ends politely when the input
//! runs out instead of looping forever.
//!
//! Run:  cargo run -p guessing_game
//! Pipe: seq 1 100 | cargo run -p guessing_game   (bash: guesses 1, 2, 3, ... until it wins)
//! Test: cargo test -p guessing_game

use colored::Colorize; // the trait that adds `.red()`, `.green()`, ... to strings
use rand::Rng; // the trait that adds `.random_range()` to random number generators
use std::cmp::Ordering;
use std::io;

const MIN: u32 = 1;
const MAX: u32 = 100;

fn main() {
    println!("Guess the number between {MIN} and {MAX}!");
    let secret_number = rand::rng().random_range(MIN..=MAX);
    let mut attempts = 0;

    loop {
        println!("Please input your guess.");

        let mut guess = String::new();
        let bytes_read = io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");
        if bytes_read == 0 {
            // 0 bytes means the input has ended: Ctrl+D (Linux), Ctrl+Z then
            // Enter (Windows), or a pipe that ran dry. Without this check the
            // empty line would fail to parse, `continue`, and loop forever.
            println!("No more input. The number was {secret_number}.");
            break;
        }

        // Shadowing: the String `guess` is replaced by a number `guess`.
        let guess: u32 = match parse_guess(&guess) {
            Some(num) => num,
            None => {
                println!(
                    "{}",
                    format!("Enter a whole number from {MIN} to {MAX}.").yellow()
                );
                continue;
            }
        };
        attempts += 1;
        println!("You guessed: {guess}");

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("{}", "Too small!".red()),
            Ordering::Greater => println!("{}", "Too big!".red()),
            Ordering::Equal => {
                println!("{}", format!("You win in {attempts} attempt(s)!").green());
                break;
            }
        }
    }
}

/// Turns one line of input into a guess: `Some(number)` if it is a whole
/// number from `MIN` to `MAX`, otherwise `None`.
///
/// `trim` removes the spaces and the newline that Enter adds (`"\n"`, or
/// `"\r\n"` on Windows). `parse` returns a `Result`, and the match guard
/// `if (MIN..=MAX).contains(&num)` only accepts numbers in range.
fn parse_guess(input: &str) -> Option<u32> {
    match input.trim().parse() {
        Ok(num) if (MIN..=MAX).contains(&num) => Some(num),
        _ => None,
    }
}

// `::` is the path separator: it reaches items inside modules, and associated
// functions on types, e.g. `String::new()` or `Ordering::Less`.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_whole_numbers_in_range() {
        assert_eq!(parse_guess("42"), Some(42));
        assert_eq!(parse_guess("1"), Some(MIN));
        assert_eq!(parse_guess("100"), Some(MAX));
    }

    #[test]
    fn ignores_the_newline_from_enter() {
        assert_eq!(parse_guess("42\n"), Some(42));
        assert_eq!(parse_guess("42\r\n"), Some(42)); // Windows
        assert_eq!(parse_guess("  7  "), Some(7));
    }

    #[test]
    fn rejects_numbers_out_of_range() {
        assert_eq!(parse_guess("0"), None);
        assert_eq!(parse_guess("101"), None);
        assert_eq!(parse_guess("4294967295"), None); // u32::MAX parses, but is out of range
    }

    #[test]
    fn rejects_things_that_are_not_whole_numbers() {
        assert_eq!(parse_guess(""), None);
        assert_eq!(parse_guess("fifty"), None);
        assert_eq!(parse_guess("-5"), None); // a u32 cannot be negative
        assert_eq!(parse_guess("3.5"), None);
        assert_eq!(parse_guess("4 2"), None);
        assert_eq!(parse_guess("99999999999"), None); // too big for a u32
    }

    #[test]
    fn the_secret_is_always_in_range() {
        let mut rng = rand::rng();
        for _ in 0..1_000 {
            let secret = rng.random_range(MIN..=MAX);
            assert!((MIN..=MAX).contains(&secret));
        }
    }
}
