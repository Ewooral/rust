//! Solution to exercise 4: FizzBuzz, the classic.
//!
//! "Both" must be checked first: 15 is a multiple of 3, so a `% 3` check
//! placed first would answer "Fizz" and never look further.
//!
//! Test: cargo test -p control_flow --features solutions --bin sol04_fizzbuzz

fn fizzbuzz(n: u32) -> String {
    if n.is_multiple_of(15) {
        String::from("FizzBuzz")
    } else if n.is_multiple_of(3) {
        String::from("Fizz")
    } else if n.is_multiple_of(5) {
        String::from("Buzz")
    } else {
        n.to_string()
    }
}

fn main() {
    for n in 1..=15 {
        println!("{}", fizzbuzz(n));
    }
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
