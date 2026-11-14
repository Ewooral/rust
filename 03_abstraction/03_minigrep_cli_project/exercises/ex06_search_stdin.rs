//! Exercise 6 (Beyond the Book): search any reader, line by line.
//!
//! minigrep reads the whole file into memory first. Real grep streams: it
//! reads one line at a time, from a file OR from standard input. Implement
//! `search_reader`, which takes anything that implements `BufRead`.
//! Because the lines are read one by one, each one is an owned String, not
//! a slice: why can't it return `&str` here?
//!
//!   Test: cargo test -p minigrep --features exercises --bin ex06_search_stdin
//!   Try:  echo "one fish`ntwo fish`nred" | cargo run -p minigrep --features exercises --bin ex06_search_stdin -- fish
//!         (bash: printf 'one fish\ntwo fish\nred\n' | cargo run ...)

#![allow(unused_variables)]

use std::io::{self, BufRead};

/// Every line containing `query`. Reading can fail, so this returns
/// io::Result: pass errors on with `?`.
fn search_reader(query: &str, reader: impl BufRead) -> io::Result<Vec<String>> {
    todo!()
}

fn main() -> io::Result<()> {
    let query = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "fish".to_string());
    for line in search_reader(&query, io::stdin().lock())? {
        println!("{line}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn searches_any_reader() -> io::Result<()> {
        // A byte slice implements BufRead, so tests need no real file.
        let input = "one fish\ntwo fish\nred fish\nblue\n".as_bytes();
        assert_eq!(
            search_reader("fish", input)?,
            ["one fish", "two fish", "red fish"]
        );
        Ok(())
    }

    #[test]
    fn handles_windows_line_endings() -> io::Result<()> {
        let input = "alpha\r\nbeta\r\n".as_bytes();
        assert_eq!(search_reader("beta", input)?, ["beta"]);
        Ok(())
    }

    #[test]
    fn empty_input() -> io::Result<()> {
        assert!(search_reader("x", "".as_bytes())?.is_empty());
        Ok(())
    }
}
