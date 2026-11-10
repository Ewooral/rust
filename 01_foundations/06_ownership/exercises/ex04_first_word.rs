//! Exercise 4: return a slice.
//!
//! Implement `first_word`: return the first word of `s` as a slice of `s`
//! (no new String). Words are separated by whitespace; leading whitespace is
//! skipped. Return "" when there is no word.
//!
//! Test: cargo test -p ownership --features exercises --bin ex04_first_word

#![allow(unused_variables)]

fn first_word(s: &str) -> &str {
    todo!("return the first word of {s:?}")
}

fn main() {
    let sentence = String::from("ownership is Rust's superpower");
    println!("{}", first_word(&sentence));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_word() {
        assert_eq!(first_word("hello"), "hello");
    }

    #[test]
    fn several_words() {
        assert_eq!(first_word("hello brave new world"), "hello");
    }

    #[test]
    fn leading_whitespace() {
        assert_eq!(first_word("   \tspaced out"), "spaced");
    }

    #[test]
    fn empty_and_blank() {
        assert_eq!(first_word(""), "");
        assert_eq!(first_word("   "), "");
    }

    #[test]
    fn works_with_a_string() {
        let owned = String::from("from a String");
        assert_eq!(first_word(&owned), "from");
    }
}
