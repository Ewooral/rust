//! Solution to exercise 4: JavaScript string positions vs Rust string positions (pass 3 tests).
//!
//! A text editor in the browser sends Rust a cursor position. JavaScript
//! counts positions in UTF-16 code units; Rust's `str` indexes are UTF-8
//! BYTES. They only agree for plain ASCII:
//!
//!     text     "a👋é"
//!     JS       a=0  👋=1,2 (two units: a surrogate pair)  é=3        length 4
//!     Rust     a=0  👋=1..5 (four bytes)                  é=5..7     len 7
//!
//! Implement:
//!   - `utf16_to_byte`: a JS index -> the byte index of the same place.
//!     The end of the string is allowed. `None` past the end, or in the
//!     middle of a character (between the two halves of a pair).
//!   - `byte_to_utf16`: the other way. `None` past the end or if
//!     `byte_index` isn't on a character boundary (`text.is_char_boundary`).
//!   - `js_slice`: what JavaScript's `text.slice(start, end)` returns, using
//!     the two functions above. `None` if either index is bad or start > end.
//!
//! Hints: `text.char_indices()` gives each char's byte index, and
//! `ch.len_utf16()` its size in UTF-16 units (1 or 2).
//!
//! Test: cargo test -p webassembly --features solutions --bin sol04_utf16_offsets

fn utf16_to_byte(text: &str, utf16_index: usize) -> Option<usize> {
    let mut units = 0;
    for (byte, ch) in text.char_indices() {
        if units == utf16_index {
            return Some(byte);
        }
        units += ch.len_utf16();
        if units > utf16_index {
            return None; // the index fell between the two halves of a pair
        }
    }
    (units == utf16_index).then_some(text.len())
}

fn byte_to_utf16(text: &str, byte_index: usize) -> Option<usize> {
    // is_char_boundary is false past the end, and true at text.len().
    if !text.is_char_boundary(byte_index) {
        return None;
    }
    Some(text[..byte_index].encode_utf16().count())
}

fn js_slice(text: &str, start: usize, end: usize) -> Option<&str> {
    if start > end {
        return None;
    }
    let (from, to) = (utf16_to_byte(text, start)?, utf16_to_byte(text, end)?);
    text.get(from..to)
}

fn main() {
    let text = "a👋é";
    println!(
        "{text:?}: JS length {}, Rust len {}",
        text.encode_utf16().count(),
        text.len()
    );
    for js_index in 0..=4 {
        println!(
            "  JS index {js_index} -> byte {:?}",
            utf16_to_byte(text, js_index)
        );
    }
    println!("  byte 5 -> JS index {:?}", byte_to_utf16(text, 5));
    println!("  JS text.slice(1, 3) = {:?}", js_slice(text, 1, 3));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_positions_are_the_same() {
        assert_eq!(utf16_to_byte("hello", 3), Some(3));
        assert_eq!(byte_to_utf16("hello", 5), Some(5), "the end is allowed");
        assert_eq!(utf16_to_byte("hello", 6), None);
        assert_eq!(js_slice("hello", 1, 4), Some("ell"));
    }

    #[test]
    fn accents_and_emoji_differ() {
        let text = "a👋é";
        assert_eq!(utf16_to_byte(text, 1), Some(1));
        assert_eq!(utf16_to_byte(text, 3), Some(5));
        assert_eq!(utf16_to_byte(text, 4), Some(7));
        assert_eq!(byte_to_utf16(text, 5), Some(3));
        assert_eq!(byte_to_utf16(text, 7), Some(4));
        assert_eq!(js_slice(text, 1, 3), Some("👋"));
        assert_eq!(js_slice("café au lait", 3, 4), Some("é"));
    }

    #[test]
    fn positions_inside_a_character_are_rejected() {
        let text = "a👋é";
        assert_eq!(utf16_to_byte(text, 2), None, "between the surrogate halves");
        assert_eq!(byte_to_utf16(text, 2), None, "inside the emoji's bytes");
        assert_eq!(byte_to_utf16(text, 8), None);
        assert_eq!(js_slice(text, 3, 1), None);
        assert_eq!(js_slice(text, 0, 2), None);
    }
}
