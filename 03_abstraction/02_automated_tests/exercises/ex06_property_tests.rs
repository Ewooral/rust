//! Exercise 6 (Beyond the Book): write property tests with proptest.
//!
//! `encode` shifts each ASCII letter 13 places (ROT13) and leaves everything
//! else alone. Instead of picking examples, state properties that hold for
//! EVERY string, and let proptest try hundreds of random ones:
//!   1. encoding twice gives back the original (ROT13 undoes itself),
//!   2. encoding never changes the number of characters,
//!   3. characters that are not ASCII letters are never changed.
//!
//! Fill in the three tests. `s in ".*"` generates random strings, and
//! `prop_assert_eq!` / `prop_assert!` are the assertions to use.
//!
//! Test: cargo test -p automated_tests --features exercises --bin ex06_property_tests

fn encode(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            'a'..='z' => (((c as u8 - b'a') + 13) % 26 + b'a') as char,
            'A'..='Z' => (((c as u8 - b'A') + 13) % 26 + b'A') as char,
            _ => c,
        })
        .collect()
}

fn main() {
    let secret = encode("Hello, Ferris!");
    println!("{secret} -> {}", encode(&secret));
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn encoding_twice_gives_the_original(s in ".*") {
            let _ = s;
            todo!("property 1");
        }

        #[test]
        fn length_in_chars_is_unchanged(s in ".*") {
            let _ = s;
            todo!("property 2");
        }

        #[test]
        fn non_letters_are_untouched(s in ".*") {
            let _ = s;
            todo!("property 3");
        }
    }
}
