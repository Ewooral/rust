//! Builds HTTP requests with the `api_design` library and prints them as
//! they would go over the wire. Errors are printed with their whole
//! `source()` chain.
//!
//! Run:  cargo run -p api_design                    (a short tour)
//!       cargo run -p api_design -- GET "http://example.org/search?q=rust"
//!       cargo run -p api_design -- POST http://localhost:8080/notes "buy milk"
//!       cargo run -p api_design -- GET http://example.org:99999/
//! Test: cargo test -p api_design

use std::error::Error;
use std::process::ExitCode;

use api_design::{Method, Request, RequestError};

/// Prints an error and every error underneath it. An APPLICATION reports
/// errors like this (or with `anyhow`); the LIBRARY just keeps the chain.
fn report(error: &dyn Error) {
    eprintln!("error: {error}");
    let mut cause = error.source();
    while let Some(inner) = cause {
        eprintln!("  caused by: {inner}");
        cause = inner.source();
    }
}

/// `METHOD URL [BODY]`, as typed on the command line.
fn from_args(args: &[String]) -> Result<Request, Box<dyn Error + Send + Sync>> {
    let [method, url, rest @ ..] = args else {
        return Err("usage: api_design [METHOD URL [BODY]]".into());
    };
    let method: Method = method.parse()?; // ParseMethodError -> Box<dyn Error>, via `From`
    let mut builder = Request::builder()
        .method(method)
        .url(url)
        .header("User-Agent", "api_design/0.1");
    if let Some(body) = rest.first() {
        builder = builder
            .header("Content-Type", "text/plain")
            .body(body.as_str());
    }
    Ok(builder.build()?) // RequestError -> Box<dyn Error + Send + Sync>
}

/// Three requests that work, and two that don't.
fn tour() -> Vec<Result<Request, RequestError>> {
    vec![
        Request::get("http://example.org/"),
        Request::builder()
            .method(Method::Post)
            .url("https://api.example.org:8443/v1/notes")
            .header("Content-Type", "application/json")
            .body(r#"{"text": "buy milk"}"#)
            .build(),
        Request::builder()
            .method(Method::Put)
            .url("http://localhost:3000/avatar.bin")
            .body(vec![0xff, 0xd8, 0xff, 0xe0])
            .build(),
        Request::get("http://example.org:99999/"),
        Request::builder()
            .method(Method::Post)
            .url("http://localhost/upload")
            .body_file("no/such/file.txt")
            .build(),
    ]
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        for (number, result) in tour().into_iter().enumerate() {
            println!("--- request {} ---", number + 1);
            match result {
                Ok(request) => println!("{request}\n"),
                Err(error) => {
                    report(&error);
                    println!();
                }
            }
        }
        return ExitCode::SUCCESS;
    }

    match from_args(&args) {
        Ok(request) => {
            println!("{request}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            report(error.as_ref());
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn the_tour_has_three_good_requests_and_two_errors() {
        let results = tour();
        assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 3);
        assert!(matches!(results[3], Err(RequestError::InvalidUrl { .. })));
        assert!(matches!(results[4], Err(RequestError::BodyFile { .. })));
    }

    #[test]
    fn command_line_requests() {
        let request = from_args(&args(&["post", "http://h/notes", "hi"])).unwrap();
        assert_eq!(request.method(), Method::Post);
        assert_eq!(request.headers().get("content-length"), Some("2"));
        let error = from_args(&args(&["FETCH", "http://h/"])).unwrap_err();
        assert_eq!(error.to_string(), "unknown HTTP method \"FETCH\"");
        assert!(from_args(&args(&["GET"])).is_err());
    }
}
