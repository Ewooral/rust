//! Testing that code panics, and tests that return Result.
//! Run: cargo test -p automated_tests --example should_panic_and_result

pub struct Guess {
    value: i32,
}

impl Guess {
    pub fn new(value: i32) -> Guess {
        if value < 1 {
            panic!("Guess value must be greater than or equal to 1, got {value}.");
        } else if value > 100 {
            panic!("Guess value must be less than or equal to 100, got {value}.");
        }
        Guess { value }
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}

fn parse_score(text: &str) -> Result<u32, std::num::ParseIntError> {
    text.trim().parse()
}

fn main() {
    println!(
        "Run me with `cargo test`: see the header. Guess 50 -> {}",
        Guess::new(50).value()
    );
    println!("parse_score(\"7\") = {:?}", parse_score("7"));
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[should_panic]: the test passes only if the code panics.
    #[test]
    #[should_panic]
    fn greater_than_100() {
        Guess::new(200);
    }

    // `expected = "..."` also checks the panic message CONTAINS that text, so
    // the test cannot pass by panicking for the wrong reason.
    #[test]
    #[should_panic(expected = "less than or equal to 100")]
    fn greater_than_100_with_reason() {
        Guess::new(200);
    }

    #[test]
    #[should_panic(expected = "greater than or equal to 1")]
    fn less_than_1() {
        Guess::new(0);
    }

    // A test may return Result<(), E>. Err fails the test, and `?` works.
    // (You can't combine this with #[should_panic].)
    #[test]
    fn parses_a_score() -> Result<(), std::num::ParseIntError> {
        let score = parse_score(" 42 ")?;
        assert_eq!(score, 42);
        Ok(())
    }

    // To check that an operation returns Err, assert on it; don't use `?`.
    #[test]
    fn rejects_text() {
        assert!(parse_score("forty-two").is_err());
    }
}
