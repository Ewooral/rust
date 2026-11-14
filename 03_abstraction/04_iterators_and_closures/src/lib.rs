//! minigrep v2 (Book 13.3): the chapter 12 program, improved with iterators.
//!
//! Compare with 03_minigrep_cli_project/src/lib.rs:
//! - `Config::build` takes the argument ITERATOR itself, so it can take
//!   ownership of each String: no more `.clone()`, no more indexing.
//! - `search` is one iterator chain: no mutable Vec, no explicit loop.

use std::env;
use std::error::Error;
use std::fs;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
}

impl Config {
    /// `impl Iterator<Item = String>`: accepts `env::args()` directly, or any
    /// other iterator of Strings (handy in tests). `mut` because next()
    /// advances it.
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next(); // skip the program name

        let query = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a query string"),
        };

        // The same thing, written with ok_or: an Option turned into a Result.
        let file_path = args.next().ok_or("Didn't get a file path")?;

        let ignore_case = env::var("IGNORE_CASE").is_ok();

        Ok(Config {
            query,
            file_path,
            ignore_case,
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?;

    let results = if config.ignore_case {
        search_case_insensitive(&config.query, &contents)
    } else {
        search(&config.query, &contents)
    };

    for line in results {
        println!("{line}");
    }
    Ok(())
}

/// Chapter 12 used a mutable Vec and a for loop. One chain says the same,
/// with less to get wrong, and runs just as fast.
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents
        .lines()
        .filter(|line| line.contains(query))
        .collect()
}

pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase();
    contents
        .lines()
        .filter(|line| line.to_lowercase().contains(&query)) // the closure captures `query`
        .collect()
}

/// Beyond the Book: a small report built from one pass of iterator adapters.
/// Returns (line count, word count, the most frequent word longer than 3 letters).
pub fn summary(contents: &str) -> (usize, usize, Option<String>) {
    use std::collections::HashMap;

    let lines = contents.lines().count();
    let words: Vec<String> = contents
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect();

    let mut counts: HashMap<&str, usize> = HashMap::new();
    for w in words.iter().filter(|w| w.chars().count() > 3) {
        *counts.entry(w.as_str()).or_insert(0) += 1;
    }
    let top = counts
        .into_iter()
        .max_by(|(wa, ca), (wb, cb)| ca.cmp(cb).then(wb.cmp(wa))) // ties: alphabetical first
        .map(|(word, _)| word.to_string());

    (lines, words.len(), top)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> impl Iterator<Item = String> {
        // A Vec's into_iter is just as good an argument iterator as env::args.
        list.iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn build_takes_ownership_from_the_iterator() {
        let config = Config::build(args(&["minigrep", "to", "poem.txt"])).unwrap();
        assert_eq!(config.query, "to");
        assert_eq!(config.file_path, "poem.txt");
    }

    #[test]
    fn build_reports_what_is_missing() {
        assert_eq!(
            Config::build(args(&["minigrep"])),
            Err("Didn't get a query string")
        );
        assert_eq!(
            Config::build(args(&["minigrep", "to"])),
            Err("Didn't get a file path")
        );
    }

    #[test]
    fn case_sensitive() {
        let contents = "Rust:\nsafe, fast, productive.\nPick three.\nDuct tape.";
        assert_eq!(vec!["safe, fast, productive."], search("duct", contents));
    }

    #[test]
    fn case_insensitive() {
        let contents = "Rust:\nsafe, fast, productive.\nPick three.\nTrust me.";
        assert_eq!(
            vec!["Rust:", "Trust me."],
            search_case_insensitive("rUsT", contents)
        );
    }

    #[test]
    fn summary_of_the_poem() {
        let poem = include_str!("../poem.txt");
        let (lines, words, top) = summary(poem);
        assert_eq!(lines, 9);
        assert_eq!(words, 43); // the lone "-" is not a word
        assert_eq!(top.as_deref(), Some("nobody"));
    }
}
