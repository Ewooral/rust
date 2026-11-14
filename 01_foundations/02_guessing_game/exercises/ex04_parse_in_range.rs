//! Exercise 4: validate a guess.
//!
//! Implement `parse_in_range`: turn one line of input into a number, but only
//! if it is a whole number from `min` to `max` (both included). Return
//! `Some(number)` when it is valid and `None` when it isn't. (`Option` is
//! explained in lesson 02/02; for now, `Some(x)` means "here it is" and
//! `None` means "nothing".)
//!
//! Hints: remember the newline that Enter adds. `parse` returns a `Result`,
//! so `match` on it. `(min..=max).contains(&n)` checks the range.
//!
//! Test: cargo test -p guessing_game --features exercises --bin ex04_parse_in_range

#![allow(unused_variables)]

fn parse_in_range(input: &str, min: u32, max: u32) -> Option<u32> {
    todo!("parse {input:?} and check it is from {min} to {max}")
}

fn main() {
    for input in ["7\n", "0\n", "eleven\n"] {
        println!("{input:?} -> {:?}", parse_in_range(input, 1, 10));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_numbers_in_range() {
        assert_eq!(parse_in_range("7", 1, 10), Some(7));
        assert_eq!(parse_in_range("1", 1, 10), Some(1));
        assert_eq!(parse_in_range("10", 1, 10), Some(10));
    }

    #[test]
    fn ignores_surrounding_whitespace() {
        assert_eq!(parse_in_range("7\n", 1, 10), Some(7));
        assert_eq!(parse_in_range("7\r\n", 1, 10), Some(7));
        assert_eq!(parse_in_range("   7 ", 1, 10), Some(7));
    }

    #[test]
    fn rejects_numbers_outside_the_range() {
        assert_eq!(parse_in_range("0", 1, 10), None);
        assert_eq!(parse_in_range("11", 1, 10), None);
    }

    #[test]
    fn rejects_text_that_is_not_a_whole_number() {
        assert_eq!(parse_in_range("", 1, 10), None);
        assert_eq!(parse_in_range("seven", 1, 10), None);
        assert_eq!(parse_in_range("-3", 1, 10), None);
        assert_eq!(parse_in_range("2.5", 1, 10), None);
    }
}
