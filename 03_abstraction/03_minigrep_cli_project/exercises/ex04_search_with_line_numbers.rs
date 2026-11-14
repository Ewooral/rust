//! Exercise 4: extend minigrep, test first.
//!
//! Real grep can print line numbers (`grep -n`). Implement
//! `search_numbered`: like `search`, but each result also carries its line
//! number, counting from 1. The tests are written; make them pass.
//! Hint: `contents.lines().enumerate()`.
//!
//! Test: cargo test -p minigrep --features exercises --bin ex04_search_with_line_numbers

#![allow(unused_variables)]

fn search_numbered<'a>(query: &str, contents: &'a str) -> Vec<(usize, &'a str)> {
    todo!()
}

fn main() {
    let poem = include_str!("../poem.txt");
    for (number, line) in search_numbered("body", poem) {
        println!("{number}: {line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_start_at_one() {
        let contents = "alpha\nbeta\ngamma";
        assert_eq!(search_numbered("alpha", contents), [(1, "alpha")]);
        assert_eq!(search_numbered("gamma", contents), [(3, "gamma")]);
    }

    #[test]
    fn blank_lines_still_count() {
        let contents = "a\n\nneedle\n\nneedle again";
        assert_eq!(
            search_numbered("needle", contents),
            [(3, "needle"), (5, "needle again")]
        );
    }

    #[test]
    fn no_match_is_empty() {
        assert!(search_numbered("x", "abc").is_empty());
    }
}
