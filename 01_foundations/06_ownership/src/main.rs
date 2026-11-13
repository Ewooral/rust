//! Mini project for Book chapter 4: a text report that never copies the text.
//!
//! Every function here borrows. The only owned `String`s are the ones we
//! deliberately create, so you can see exactly where allocation happens.
//!
//! Run: cargo run -p ownership    Test: cargo test -p ownership

/// Number of whitespace-separated words. Reads only, so `&str` is enough.
fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

/// The longest word, as a slice that borrows from `text`.
/// Ties go to the first one. Returns "" for empty text.
fn longest_word(text: &str) -> &str {
    let mut longest = "";
    for word in text.split_whitespace() {
        if word.chars().count() > longest.chars().count() {
            longest = word;
        }
    }
    longest  
}

/// Capitalises the first letter of every word, in place.
/// Mutates the caller's data, so it needs `&mut`.
fn capitalise_each(words: &mut [String]) {
    for word in words.iter_mut() {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            *word = first.to_uppercase().chain(chars).collect();
        }
    }
}

fn main() {
    let text = String::from("ownership makes memory safety a compile time guarantee");

    println!("text: {text}");
    println!("words: {}", word_count(&text));
    println!("longest: {}", longest_word(&text));

    // Here we choose to allocate: one owned String per word.
    let mut words: Vec<String> = text.split_whitespace().map(String::from).collect();
    capitalise_each(&mut words);
    println!("title case: {}", words.join(" "));

    // `text` was only ever borrowed, so it is still ours.
    println!("original still intact: {text}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_words() {
        assert_eq!(word_count("  one two\tthree \n"), 3);
        assert_eq!(word_count(""), 0);
    }

    #[test]
    fn longest_word_borrows_from_input() {
        let text = String::from("a bb ccc dd");
        let longest = longest_word(&text);
        assert_eq!(longest, "ccc");
        // The slice points into `text`'s buffer: no copy was made.
        assert!(text.as_ptr() <= longest.as_ptr());
    }

    #[test]
    fn longest_word_counts_characters_not_bytes() {
        assert_eq!(longest_word("héé abcd"), "abcd");
    }

    #[test]
    fn capitalises_in_place() {
        let mut words = vec![String::from("rust"), String::from("éclair"), String::new()];
        capitalise_each(&mut words);
        assert_eq!(words, ["Rust", "Éclair", ""]);
    }
}
