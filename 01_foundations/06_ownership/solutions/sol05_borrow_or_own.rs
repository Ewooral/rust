//! SOLUTION to exercise 5: borrow, borrow mutably, or take ownership.
//!
//! Each signature is already the right choice. Implement the bodies, then read
//! the signatures again and explain to yourself why each one was chosen.
//!
//! Test: cargo test -p ownership --features solutions --bin sol05_borrow_or_own

/// Reads only: a shared borrow of a slice is enough.
fn total_len(words: &[String]) -> usize {
    words.iter().map(|w| w.len()).sum()
}

/// Changes the caller's String in place: needs a mutable borrow.
fn shout(s: &mut String) {
    *s = s.to_uppercase();
}

/// Consumes the words and builds something new: takes ownership.
/// Joins the words with single spaces and ends with a full stop.
fn into_sentence(words: Vec<String>) -> String {
    let mut sentence = words.join(" ");
    sentence.push('.');
    sentence
}

fn main() {
    let words = vec![
        String::from("rust"),
        String::from("is"),
        String::from("fun"),
    ];
    println!("total length: {}", total_len(&words));
    let mut sentence = into_sentence(words);
    shout(&mut sentence);
    println!("{sentence}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words() -> Vec<String> {
        vec!["rust".into(), "is".into(), "fun".into()]
    }

    #[test]
    fn total_len_counts_bytes_of_all_words() {
        assert_eq!(total_len(&words()), 9);
        assert_eq!(total_len(&[]), 0);
    }

    #[test]
    fn shout_uppercases_in_place() {
        let mut s = String::from("quiet please");
        shout(&mut s);
        assert_eq!(s, "QUIET PLEASE");
    }

    #[test]
    fn into_sentence_joins_and_ends_with_full_stop() {
        assert_eq!(into_sentence(words()), "rust is fun.");
        assert_eq!(into_sentence(vec![]), ".");
    }
}
