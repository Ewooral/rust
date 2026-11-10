//! Working with strings.

/// Splits `s` into two-character groups, padding an odd tail with `_`.
///
/// Works on characters, not bytes, so non-ASCII text is safe.
///
/// ```
/// use collections::strings::group_in_pairs;
/// assert_eq!(group_in_pairs("hello"), vec!["he", "ll", "o_"]);
/// ```
pub fn group_in_pairs(s: &str) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    chars
        .chunks(2)
        .map(|pair| match pair {
            [a, b] => format!("{a}{b}"),
            [a] => format!("{a}_"),
            _ => unreachable!("chunks(2) yields 1 or 2 items"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_even_length() {
        assert_eq!(group_in_pairs("rust"), vec!["ru", "st"]);
    }

    #[test]
    fn pads_odd_length() {
        assert_eq!(group_in_pairs("hello"), vec!["he", "ll", "o_"]);
        assert_eq!(group_in_pairs("a"), vec!["a_"]);
    }

    #[test]
    fn empty_string_gives_no_groups() {
        assert!(group_in_pairs("").is_empty());
    }

    #[test]
    fn handles_multibyte_characters() {
        assert_eq!(group_in_pairs("héllo"), vec!["hé", "ll", "o_"]);
    }
}
