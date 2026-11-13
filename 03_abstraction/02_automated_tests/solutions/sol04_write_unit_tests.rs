//! Solution to exercise 4: write the tests yourself.
//!
//! `fizzbuzz` is correct. Each test below is a stub: replace the `todo!` with
//! assertions that check what the test's NAME promises. Use assert_eq! with
//! at least two inputs per test. A test that checks nothing proves nothing.
//!
//! Test: cargo test -p automated_tests --features solutions --bin sol04_write_unit_tests

/// "Fizz" for multiples of 3, "Buzz" for multiples of 5, "FizzBuzz" for
/// multiples of both, otherwise the number itself.
fn fizzbuzz(n: u32) -> String {
    match (n % 3, n % 5) {
        (0, 0) => "FizzBuzz".to_string(),
        (0, _) => "Fizz".to_string(),
        (_, 0) => "Buzz".to_string(),
        _ => n.to_string(),
    }
}

fn main() {
    let line: Vec<String> = (1..=15).map(fizzbuzz).collect();
    println!("{}", line.join(" "));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_numbers_are_returned_as_text() {
        assert_eq!(fizzbuzz(1), "1");
        assert_eq!(fizzbuzz(7), "7");
    }

    #[test]
    fn multiples_of_three_are_fizz() {
        assert_eq!(fizzbuzz(3), "Fizz");
        assert_eq!(fizzbuzz(99), "Fizz");
    }

    #[test]
    fn multiples_of_five_are_buzz() {
        assert_eq!(fizzbuzz(5), "Buzz");
        assert_eq!(fizzbuzz(100), "Buzz");
    }

    #[test]
    fn multiples_of_fifteen_are_fizzbuzz() {
        assert_eq!(fizzbuzz(15), "FizzBuzz");
        assert_eq!(fizzbuzz(45), "FizzBuzz");
    }
}
