//! Solution to exercise 5 (from the Book's chapter 8 summary): Pig Latin.
//!
//! - A word starting with a consonant moves that first letter to the end and
//!   adds "ay" after a hyphen: "first" -> "irst-fay".
//! - A word starting with a vowel (a e i o u) just adds "-hay":
//!   "apple" -> "apple-hay".
//! - Keep in mind the details about UTF-8: take the first CHAR, not byte 0.
//!   "été" starts with 'é', which is not in our vowel list, so it is a
//!   consonant here: "été" -> "té-éay".
//! - `sentence` converts every whitespace-separated word, joined by one space.
//!
//! Test: cargo test -p collections --features solutions --bin sol05_pig_latin

fn pig_latin(word: &str) -> String {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    if "aeiou".contains(first) {
        format!("{word}-hay")
    } else {
        // `chars.as_str()` is the rest of the word after the first char.
        format!("{}-{first}ay", chars.as_str())
    }
}

fn sentence(text: &str) -> String {
    text.split_whitespace()
        .map(pig_latin)
        .collect::<Vec<_>>()
        .join(" ")
}

fn main() {
    println!("{}", sentence("the quick brown fox eats an apple"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consonant_words() {
        assert_eq!(pig_latin("first"), "irst-fay");
        assert_eq!(pig_latin("rust"), "ust-ray");
        assert_eq!(pig_latin("été"), "té-éay");
    }

    #[test]
    fn vowel_words() {
        assert_eq!(pig_latin("apple"), "apple-hay");
        assert_eq!(pig_latin("under"), "under-hay");
    }

    #[test]
    fn sentences() {
        assert_eq!(sentence("hello  world"), "ello-hay orld-way");
        assert_eq!(sentence(""), "");
    }
}
