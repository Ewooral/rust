//! Exercise 5: the hint.
//!
//! Implement `hint`: compare `guess` with `secret` and return
//! "Too small!", "Too big!" or "You win!". Use `cmp` and a `match` on the
//! `Ordering`, like the game does. You'll need `use std::cmp::Ordering;`.
//!
//! Test: cargo test -p guessing_game --features exercises --bin ex05_hint

#![allow(unused_variables)]

fn hint(guess: u32, secret: u32) -> &'static str {
    todo!("compare {guess} with {secret}")
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
