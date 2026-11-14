//! Solution to exercise 4: validate a guess.
//!
//! Test: cargo test -p guessing_game --features solutions --bin sol04_parse_in_range

fn parse_in_range(input: &str, min: u32, max: u32) -> Option<u32> {
    match input.trim().parse() {
        // A match guard: this arm only matches if the condition is true too.
        Ok(n) if (min..=max).contains(&n) => Some(n),
        _ => None,
    }
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
