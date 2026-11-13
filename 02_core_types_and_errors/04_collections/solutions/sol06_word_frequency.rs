//! Solution to exercise 6: count words with a HashMap, then rank them with a Vec.
//!
//! - `word_counts`: lowercase every word and strip punctuation from its ends
//!   (`trim_matches(|c: char| !c.is_alphanumeric())`). Skip words that become
//!   empty. Use the entry API.
//! - `top_words`: the `n` most frequent words, highest count first. Ties are
//!   broken alphabetically, so the result is always the same.
//!
//! Test: cargo test -p collections --features solutions --bin sol06_word_frequency

use std::collections::HashMap;

fn word_counts(text: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for raw in text.split_whitespace() {
        let word = raw
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();
        if word.is_empty() {
            continue;
        }
        *counts.entry(word).or_insert(0) += 1;
    }
    counts
}

fn top_words(text: &str, n: usize) -> Vec<(String, usize)> {
    let mut ranked: Vec<(String, usize)> = word_counts(text).into_iter().collect();
    // Count descending, then word ascending.
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked.truncate(n);
    ranked
}

fn main() {
    let text = "The cat sat. The cat ran! A dog sat, and the dog barked.";
    for (word, count) in top_words(text, 3) {
        println!("{word}: {count}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_ignore_case_and_punctuation() {
        let counts = word_counts("Hello, hello! HELLO? world...");
        assert_eq!(counts.get("hello"), Some(&3));
        assert_eq!(counts.get("world"), Some(&1));
        assert_eq!(counts.len(), 2);
    }

    #[test]
    fn standalone_punctuation_is_not_a_word() {
        assert!(word_counts(" - ... !").is_empty());
    }

    #[test]
    fn top_words_ranked_with_ties_alphabetical() {
        let text = "b a c b a b";
        assert_eq!(
            top_words(text, 2),
            vec![("b".to_string(), 3), ("a".to_string(), 2)]
        );
        assert_eq!(
            top_words("y x", 5),
            vec![("x".to_string(), 1), ("y".to_string(), 1)]
        );
        assert!(top_words("", 3).is_empty());
    }
}
