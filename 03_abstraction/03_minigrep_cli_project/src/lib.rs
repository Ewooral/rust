//! minigrep's library crate: everything that can be tested lives here.
//!
//! `src/main.rs` only collects the arguments, calls into this library, and
//! turns errors into a message on stderr plus an exit code.

use std::env;
use std::error::Error;
use std::fs;

/// Everything the user asked for, gathered into one value.
#[derive(Debug, PartialEq)]
pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
}

impl Config {
    /// Builds a Config from the program's arguments.
    ///
    /// `args[0]` is the program's own name, so the query is `args[1]` and the
    /// file is `args[2]`. Case-insensitive search is switched on by setting
    /// the `IGNORE_CASE` environment variable to anything.
    ///
    /// Returns an error message (not a panic) if arguments are missing:
    /// a user typing too little is expected, not a bug.
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }
        let query = args[1].clone();
        let file_path = args[2].clone();
        let ignore_case = env::var("IGNORE_CASE").is_ok();
        Ok(Config {
            query,
            file_path,
            ignore_case,
        })
    }
}

/// Runs the search and prints every matching line.
///
/// `Box<dyn Error>`: any kind of error (here, a file that can't be read)
/// goes back to `main`, which decides how to report it.
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

/// Every line of `contents` that contains `query`.
///
/// The lifetime `'a` says the returned slices borrow from `contents`, not
/// from `query`: the results stay valid as long as the text does.
///
/// ```
/// let lines = minigrep::search("duct", "Rust:\nsafe, fast, productive.\nPick three.");
/// assert_eq!(lines, ["safe, fast, productive."]);
/// ```
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new();
    for line in contents.lines() {
        if line.contains(query) {
            results.push(line);
        }
    }
    results
}

/// Like `search`, ignoring upper and lower case.
pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase(); // a new String, so we pass &query below
    let mut results = Vec::new();
    for line in contents.lines() {
        if line.to_lowercase().contains(&query) {
            results.push(line);
        }
    }
    results
}

// The Book builds `search` test first (TDD): these tests were written before
// the function bodies, failed, and then drove the implementation.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_sensitive() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Duct tape.";
        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }

    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";
        assert_eq!(
            vec!["Rust:", "Trust me."],
            search_case_insensitive(query, contents)
        );
    }

    #[test]
    fn no_match_gives_an_empty_list() {
        assert!(search("zebra", "no stripes here").is_empty());
    }

    #[test]
    fn build_reads_query_and_path() {
        let args: Vec<String> = ["minigrep", "to", "poem.txt"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let config = Config::build(&args).expect("enough arguments");
        assert_eq!(config.query, "to");
        assert_eq!(config.file_path, "poem.txt");
    }

    #[test]
    fn build_rejects_missing_arguments() {
        let args = vec!["minigrep".to_string()];
        assert_eq!(Config::build(&args), Err("not enough arguments"));
    }
}
