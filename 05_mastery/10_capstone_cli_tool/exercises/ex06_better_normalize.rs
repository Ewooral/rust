//! Exercise 6: smarter grouping. logsum's `normalize` turns every run of
//! digits into `#`, so "payment 991 failed" and "payment 17 failed" are
//! counted together. Real messages have other changing parts that should
//! be grouped too. Write `normalize_more`, applying these rules:
//!
//! 1. Text in double quotes becomes `"<str>"`, even if it has spaces:
//!    `user "Ama Mensah" not found`  ->  `user "<str>" not found`
//! 2. Then, word by word (words are separated by single spaces; keep the
//!    spacing exactly as it was). Ignore punctuation around a word, one of
//!    , . ; : ( ) [ ] { } and keep it in the output.
//!    - A UUID (36 characters: hex digits with `-` at positions 8, 13, 18
//!      and 23) becomes `<uuid>`:
//!      `job (3f2b8c1e-9a4d-4e2f-8b1a-0c9d7e6f5a4b) done` -> `job (<uuid>) done`
//!    - `0x` followed by one or more hex digits becomes `<hex>`:
//!      `fault at 0x7ffe3a10` -> `fault at <hex>`
//!    - Anything else: digit runs become `#`, as before (call the library's
//!      `normalize` on the word).
//!
//! Test: cargo test -p capstone_cli_tool --features exercises --bin ex06_better_normalize

#![allow(unused_variables, dead_code)]

use capstone_cli_tool::normalize;

/// Rule 1: every `"..."` becomes `"<str>"`. An unclosed quote is left as it is.
fn replace_quoted(message: &str) -> String {
    todo!()
}

/// Whether `word` is a UUID such as 3f2b8c1e-9a4d-4e2f-8b1a-0c9d7e6f5a4b.
fn is_uuid(word: &str) -> bool {
    todo!()
}

/// Whether `word` is 0x followed by at least one hex digit.
fn is_hex(word: &str) -> bool {
    todo!()
}

/// All the rules together.
fn normalize_more(message: &str) -> String {
    todo!()
}

fn main() {
    for message in [
        "user \"Ama Mensah\" not found",
        "job (3f2b8c1e-9a4d-4e2f-8b1a-0c9d7e6f5a4b) done",
        "segfault at 0x7ffe3a10 in worker 3",
        "retry 2 of 5",
    ] {
        println!("{message:50} -> {}", normalize_more(message));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted_text_is_replaced() {
        assert_eq!(
            replace_quoted("user \"Ama Mensah\" not found"),
            "user \"<str>\" not found"
        );
        assert_eq!(
            replace_quoted("a \"b\" c \"d e\""),
            "a \"<str>\" c \"<str>\""
        );
        assert_eq!(replace_quoted("unclosed \"quote"), "unclosed \"quote");
        assert_eq!(replace_quoted("no quotes"), "no quotes");
    }

    #[test]
    fn recognises_uuids_and_hex() {
        assert!(is_uuid("3f2b8c1e-9a4d-4e2f-8b1a-0c9d7e6f5a4b"));
        assert!(is_uuid("3F2B8C1E-9A4D-4E2F-8B1A-0C9D7E6F5A4B"));
        assert!(!is_uuid("3f2b8c1e-9a4d-4e2f-8b1a-0c9d7e6f5a4")); // too short
        assert!(!is_uuid("3f2b8c1e99a4d-4e2f-8b1a-0c9d7e6f5a4b")); // no dash at 8
        assert!(!is_uuid("zf2b8c1e-9a4d-4e2f-8b1a-0c9d7e6f5a4b")); // not hex
        assert!(is_hex("0x7ffe3a10"));
        assert!(is_hex("0xFF"));
        assert!(!is_hex("0x"));
        assert!(!is_hex("0xZZ"));
        assert!(!is_hex("7ffe"));
    }

    #[test]
    fn all_rules_together() {
        assert_eq!(
            normalize_more("user \"Ama Mensah\" not found"),
            "user \"<str>\" not found"
        );
        assert_eq!(
            normalize_more("job (3f2b8c1e-9a4d-4e2f-8b1a-0c9d7e6f5a4b) done"),
            "job (<uuid>) done"
        );
        assert_eq!(
            normalize_more("segfault at 0x7ffe3a10, worker 3"),
            "segfault at <hex>, worker #"
        );
        assert_eq!(normalize_more("retry 2 of 5"), "retry # of #");
        assert_eq!(normalize_more("two  spaces kept"), "two  spaces kept");
    }

    #[test]
    fn same_shape_messages_now_match() {
        assert_eq!(
            normalize_more("order 5e0c1f7a-1b2c-4d3e-8f90-a1b2c3d4e5f6 for \"Kofi\" failed"),
            normalize_more("order 0d9e8f7a-6b5c-4d3e-2f10-0a9b8c7d6e5f for \"Esi Owusu\" failed"),
        );
        // The library's normalize alone can't do it:
        assert_ne!(normalize("for \"Kofi\""), normalize("for \"Esi Owusu\""));
    }
}
