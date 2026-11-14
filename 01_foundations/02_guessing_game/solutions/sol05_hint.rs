//! Solution to exercise 5: the hint.
//!
//! Test: cargo test -p guessing_game --features solutions --bin sol05_hint

use std::cmp::Ordering;

fn hint(guess: u32, secret: u32) -> &'static str {
    // `match` is an expression: its value is the value of the arm that ran,
    // and that value is what the function returns.
    match guess.cmp(&secret) {
        Ordering::Less => "Too small!",
        Ordering::Greater => "Too big!",
        Ordering::Equal => "You win!",
    }
}

fn main() {
    for guess in [20, 80, 50] {
        println!("{guess}: {}", hint(guess, 50));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_small() {
        assert_eq!(hint(1, 50), "Too small!");
        assert_eq!(hint(49, 50), "Too small!");
    }

    #[test]
    fn too_big() {
        assert_eq!(hint(51, 50), "Too big!");
        assert_eq!(hint(100, 50), "Too big!");
    }

    #[test]
    fn exactly_right() {
        assert_eq!(hint(50, 50), "You win!");
    }
}
