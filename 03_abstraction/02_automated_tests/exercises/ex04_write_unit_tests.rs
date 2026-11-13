//! Exercise 4: write the tests yourself.
//!
//! `fizzbuzz` is correct. Each test below is a stub: replace the `todo!` with
//! assertions that check what the test's NAME promises. Use assert_eq! with
//! at least two inputs per test. A test that checks nothing proves nothing.
//!
//! Test: cargo test -p automated_tests --features exercises --bin ex04_write_unit_tests

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
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn plain_numbers_are_returned_as_text() {
        todo!("write this test");
    }

    #[test]
    fn multiples_of_three_are_fizz() {
        todo!("write this test");
    }

    #[test]
    fn multiples_of_five_are_buzz() {
        todo!("write this test");
    }

    #[test]
    fn multiples_of_fifteen_are_fizzbuzz() {
        todo!("write this test");
    }
}
