//! Exercise 4: FizzBuzz, the classic.
//!
//! Implement `fizzbuzz`: for a multiple of 3 return "Fizz", for a multiple
//! of 5 "Buzz", for a multiple of both "FizzBuzz", otherwise the number
//! itself as text (`n.to_string()`). Then make `main` print 1 to 15 with a
//! `for` loop.
//!
//! Hints: `n.is_multiple_of(3)` is `true` when 3 divides `n`. The ORDER of
//! your `if` / `else if` checks matters: which one must come first?
//!
//! Test: cargo test -p control_flow --features exercises --bin ex04_fizzbuzz

#![allow(unused_variables)]

fn fizzbuzz(n: u32) -> String {
    todo!("Fizz, Buzz, FizzBuzz or the number, for {n}")
}

fn main() {
    // TODO: print fizzbuzz(n) for every n from 1 to 15, including 15.
    println!("{}", fizzbuzz(15));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_numbers() {
        assert_eq!(fizzbuzz(1), "1");
        assert_eq!(fizzbuzz(7), "7");
    }

    #[test]
    fn fizz_and_buzz() {
        assert_eq!(fizzbuzz(3), "Fizz");
        assert_eq!(fizzbuzz(9), "Fizz");
        assert_eq!(fizzbuzz(5), "Buzz");
        assert_eq!(fizzbuzz(20), "Buzz");
    }

    #[test]
    fn fizzbuzz_for_both() {
        assert_eq!(fizzbuzz(15), "FizzBuzz");
        assert_eq!(fizzbuzz(45), "FizzBuzz");
    }
}
