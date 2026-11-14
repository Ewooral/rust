//! Solution to exercise 6: implement a word counter that is correct AND allocation-lean.
//!
//! `top_words(text, n)` returns the `n` most frequent words, most frequent
//! first; words with the same count are in alphabetical order. A word is a
//! run of ASCII letters (`char::is_ascii_alphabetic`); anything else
//! separates words. Words are case-sensitive ("The" and "the" differ).
//!
//! Budget: at most 3 trips to the allocator for the whole call, however
//! long the text. Hints, one per thing this lesson taught:
//! - return `&str` slices of `text`: no String per word;
//! - `HashMap::with_capacity(..)`: a growing map re-allocates each time it
//!   doubles. An upper bound for the number of distinct words is fine;
//! - `sort_unstable_by` sorts in place; the stable `sort_by` may allocate
//!   a scratch buffer (allowed here, but know that it does);
//! - `truncate(n)` after sorting.
//!
//! Test: cargo test -p performance --features solutions --bin sol06_fast_word_count

use std::collections::HashMap;

use performance::alloc_counter::CountingAlloc;

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

fn top_words(text: &str, n: usize) -> Vec<(&str, usize)> {
    // A word needs at least one letter and one separator, so there can't be
    // more than len / 2 + 1 distinct words: one allocation, no re-hashing.
    let mut counts: HashMap<&str, usize> = HashMap::with_capacity(text.len() / 2 + 1);
    for word in text.split(|c: char| !c.is_ascii_alphabetic()) {
        if !word.is_empty() {
            *counts.entry(word).or_insert(0) += 1; // the key borrows from `text`
        }
    }
    // Allocation 2: exactly sized, because the map knows its length.
    let mut ranked: Vec<(&str, usize)> = counts.into_iter().collect();
    // In place, no scratch buffer. Words are unique, so unstable is safe.
    ranked.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    ranked.truncate(n);
    ranked
}

fn main() {
    let text = "the cat and the hat and the bat";
    println!("{:?}", top_words(text, 3));
}

#[cfg(test)]
mod tests {
    use super::*;
    use performance::alloc_counter::count_allocations;

    #[test]
    fn counts_and_orders_words() {
        let text = "the cat and the hat and the bat";
        assert_eq!(top_words(text, 2), [("the", 3), ("and", 2)]);
        assert_eq!(
            top_words(text, 10),
            [("the", 3), ("and", 2), ("bat", 1), ("cat", 1), ("hat", 1)]
        );
    }

    #[test]
    fn anything_but_letters_separates_words() {
        assert_eq!(
            top_words("one,two;;two\nThree 3 two", 9),
            [("two", 3), ("Three", 1), ("one", 1)]
        );
        assert!(top_words("  42 ... !", 5).is_empty());
        assert!(top_words("", 5).is_empty());
    }

    #[test]
    fn stays_within_the_allocation_budget() {
        // 5 000 words, 676 different ones: "aa", "ab", ... "zz".
        let words: Vec<String> = (0..5_000u32)
            .map(|i| {
                let k = (i * i) % 676;
                let first = (b'a' + (k / 26) as u8) as char;
                let second = (b'a' + (k % 26) as u8) as char;
                format!("{first}{second}")
            })
            .collect();
        let text = words.join(" ");
        let (top, stats) = count_allocations(|| top_words(&text, 5));
        assert_eq!(top.len(), 5);
        assert!(top[0].1 >= top[4].1);
        assert!(
            stats.allocations <= 3,
            "{} allocations, budget is 3",
            stats.allocations
        );
    }
}
