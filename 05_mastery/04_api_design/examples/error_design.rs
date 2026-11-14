//! Error design for a library: one error enum with `thiserror`, details in
//! each variant, the cause kept as `source()`, and how an application
//! prints the whole chain.
//!
//! Run: cargo run -p api_design --example error_design

use std::error::Error;
use std::num::ParseIntError;

/// A library's error: callers can `match` on it, and every variant says
/// what went wrong in words (via `Display`) AND in data (its fields).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
enum ConfigError {
    /// `#[from]` writes `impl From<std::io::Error> for ConfigError`, so `?`
    /// converts, and makes the io::Error the `source()`.
    #[error("could not read the config file")]
    Io(#[from] std::io::Error),

    /// `#[source]` keeps the cause without a `From` impl: this variant
    /// needs more context (which line) than `?` could fill in.
    #[error("line {line}: {key} must be a number")]
    NotANumber {
        line: usize,
        key: String,
        #[source]
        source: ParseIntError,
    },

    /// No source: this error starts here.
    #[error("missing required key `{0}`")]
    Missing(&'static str),
}

/// Messages: lower case, no full stop, and NOT repeating the source's text,
/// because whoever prints the chain prints the source too.
fn parse_port(text: &str) -> Result<u16, ConfigError> {
    for (index, line) in text.lines().enumerate() {
        if let Some(value) = line.strip_prefix("port=") {
            return value.parse().map_err(|source| ConfigError::NotANumber {
                line: index + 1,
                key: "port".into(),
                source,
            });
        }
    }
    Err(ConfigError::Missing("port"))
}

fn load_port(path: &str) -> Result<u16, ConfigError> {
    let text = std::fs::read_to_string(path)?; // io::Error -> ConfigError, by #[from]
    parse_port(&text)
}

/// What an APPLICATION does with any error: print it and every cause.
/// (`anyhow` does this for you with `{:#}`; see lesson 02/05.)
fn report(error: &dyn Error) -> String {
    let mut text = error.to_string();
    let mut cause = error.source();
    while let Some(inner) = cause {
        text += &format!("\n    caused by: {inner}");
        cause = inner.source();
    }
    text
}

fn main() {
    println!("ok:      {:?}", parse_port("host=x\nport=8080"));

    for result in [
        parse_port("host=x\nport=eighty"),
        parse_port("host=x"),
        load_port("no/such/config.txt"),
    ] {
        let error = result.unwrap_err();
        // A caller that can handle one case matches on it...
        let advice = match &error {
            ConfigError::Missing(key) => format!("add `{key}=…` to the file"),
            ConfigError::NotANumber { line, .. } => format!("fix line {line}"),
            ConfigError::Io(io) if io.kind() == std::io::ErrorKind::NotFound => {
                "create the file".to_string()
            }
            _ => "no idea".to_string(), // #[non_exhaustive] would require this in other crates
        };
        // ...and everyone can print it with its causes.
        println!("error:   {}\n  advice: {advice}", report(&error));
    }

    // The library's own errors chain the same way.
    let error = api_design::Request::get("http://example.org:99999/").unwrap_err();
    println!("library: {}", report(&error));

    // Library errors should be Send + Sync + 'static, so applications can
    // box them and send them across threads:
    let boxed: Box<dyn Error + Send + Sync> = Box::new(error);
    let handle = std::thread::spawn(move || boxed.to_string());
    println!("from another thread: {}", handle.join().unwrap());
}
