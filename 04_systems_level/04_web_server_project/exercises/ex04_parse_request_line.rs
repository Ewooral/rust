//! Exercise 4: parse an HTTP request line.
//!
//! The Book's server compares the whole line with `"GET / HTTP/1.1"`. A real
//! server splits it into parts. Implement `parse_request_line`:
//!
//!   "GET /search?q=rust HTTP/1.1"
//!    ^^^ ^^^^^^^ ^^^^^^ ^^^^^^^^
//!    method  path  query   version
//!
//! Rules, in this order:
//!   - an empty (or all-space) line          -> Err(ParseError::Empty)
//!   - not exactly three parts               -> Err(ParseError::WrongPartCount(n))
//!   - the version doesn't start with HTTP/  -> Err(ParseError::BadVersion(version))
//!   - the target doesn't start with /       -> Err(ParseError::PathMustStartWithSlash)
//!   - otherwise split the target at the first '?' into path and query.
//!
//! The result BORROWS from the line (no copying), hence the lifetime.
//! Hints: `split_whitespace`, `collect::<Vec<_>>()`, `split_once('?')`.
//!
//! Test: cargo test -p web_server --features exercises --bin ex04_parse_request_line

#![allow(unused_variables)]

#[derive(Debug, PartialEq, Eq)]
struct RequestLine<'a> {
    method: &'a str,
    path: &'a str,
    query: Option<&'a str>,
    version: &'a str,
}

#[derive(Debug, PartialEq, Eq)]
enum ParseError {
    Empty,
    WrongPartCount(usize),
    BadVersion(String),
    PathMustStartWithSlash,
}

fn parse_request_line(line: &str) -> Result<RequestLine<'_>, ParseError> {
    todo!()
}

fn main() {
    for line in ["GET / HTTP/1.1", "GET /search?q=rust HTTP/1.1", "HELLO"] {
        println!("{line:?} -> {:?}", parse_request_line(line));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_method_path_and_version() {
        assert_eq!(
            parse_request_line("GET / HTTP/1.1"),
            Ok(RequestLine {
                method: "GET",
                path: "/",
                query: None,
                version: "HTTP/1.1",
            })
        );
        let post = parse_request_line("POST /orders HTTP/1.0").unwrap();
        assert_eq!((post.method, post.path), ("POST", "/orders"));
    }

    #[test]
    fn splits_off_the_query_string() {
        let line = parse_request_line("GET /search?q=rust&page=2 HTTP/1.1").unwrap();
        assert_eq!(line.path, "/search");
        assert_eq!(line.query, Some("q=rust&page=2"));
    }

    #[test]
    fn rejects_bad_lines() {
        assert_eq!(parse_request_line(""), Err(ParseError::Empty));
        assert_eq!(parse_request_line("   "), Err(ParseError::Empty));
        assert_eq!(
            parse_request_line("GET /"),
            Err(ParseError::WrongPartCount(2))
        );
        assert_eq!(
            parse_request_line("GET / HTTP/1.1 extra"),
            Err(ParseError::WrongPartCount(4))
        );
        assert_eq!(
            parse_request_line("GET / FTP/1"),
            Err(ParseError::BadVersion(String::from("FTP/1")))
        );
        assert_eq!(
            parse_request_line("GET index.html HTTP/1.1"),
            Err(ParseError::PathMustStartWithSlash)
        );
    }
}
