//! JSON or TOML: one interface, and errors that point at the line.
//!
//! Serde's big idea is that *your types don't care about the format*. The
//! same `Settings` struct is read by `serde_json` and by `toml`, because
//! both implement Serde's `Deserializer` trait. This module just picks the
//! right crate, and turns each crate's error into one [`ParseError`] with a
//! line, a column and a message.

use std::fmt;
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;

/// A text format the settings can be stored in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// `.json`, via `serde_json`.
    Json,
    /// `.toml`, via the `toml` crate.
    Toml,
}

impl Format {
    /// Picks the format from a file's extension.
    pub fn from_path(path: &Path) -> Option<Format> {
        match path.extension()?.to_str()? {
            "json" => Some(Format::Json),
            "toml" => Some(Format::Toml),
            _ => None,
        }
    }

    /// Reads any `T` from `text`.
    ///
    /// `DeserializeOwned` (not `Deserialize<'de>`) because the result must
    /// not borrow from `text`: we read the same text more than once while
    /// migrating, and the caller may drop it straight after.
    ///
    /// # Errors
    ///
    /// A [`ParseError`] with the position of the problem.
    pub fn parse<T: DeserializeOwned>(self, text: &str) -> Result<T, ParseError> {
        match self {
            Format::Json => serde_json::from_str(text).map_err(|err| {
                // serde_json's Display adds " at line L column C" to the
                // message; we keep the numbers separately, so cut it off.
                let full = err.to_string();
                let suffix = format!(" at line {} column {}", err.line(), err.column());
                let message = full.strip_suffix(&suffix).unwrap_or(&full).to_string();
                ParseError::new(self, err.line(), err.column(), message)
            }),
            Format::Toml => toml::from_str(text).map_err(|err| {
                // toml gives a byte range instead; turn it into line/column.
                let (line, column) = match err.span() {
                    Some(span) => line_and_column(text, span.start),
                    None => (0, 0),
                };
                ParseError::new(self, line, column, err.message().to_string())
            }),
        }
    }

    /// Writes `value` in this format, pretty-printed.
    ///
    /// # Errors
    ///
    /// The format's message, for values it can't represent (TOML has no
    /// `null`, and the top level must be a table).
    pub fn write<T: Serialize>(self, value: &T) -> Result<String, String> {
        match self {
            Format::Json => serde_json::to_string_pretty(value).map_err(|err| err.to_string()),
            Format::Toml => toml::to_string_pretty(value).map_err(|err| err.to_string()),
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Format::Json => "JSON",
            Format::Toml => "TOML",
        })
    }
}

/// 1-based line and column of byte `offset` in `text`.
fn line_and_column(text: &str, offset: usize) -> (usize, usize) {
    let before = &text[..offset.min(text.len())];
    let line = before.matches('\n').count() + 1;
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    (line, before[line_start..].chars().count() + 1)
}

/// A file that couldn't be read: where, and why.
///
/// `line` and `column` start at 1. They are 0 when the format couldn't
/// tell (an untagged or internally tagged enum buffers its input first,
/// and loses the position).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Which format was being read.
    pub format: Format,
    /// 1-based line, or 0 if unknown.
    pub line: usize,
    /// 1-based column, or 0 if unknown.
    pub column: usize,
    /// What went wrong, without the position.
    pub message: String,
}

impl ParseError {
    fn new(format: Format, line: usize, column: usize, message: String) -> ParseError {
        ParseError {
            format,
            line,
            column,
            message,
        }
    }

    /// The offending line of `text` with a `^` under the column, like a
    /// compiler error. Empty if the position is unknown.
    pub fn snippet(&self, text: &str) -> String {
        let Some(source_line) = text.lines().nth(self.line.wrapping_sub(1)) else {
            return String::new();
        };
        let number = self.line.to_string();
        let pad = " ".repeat(number.len());
        let caret = " ".repeat(self.column.saturating_sub(1));
        format!("{pad} |\n{number} | {source_line}\n{pad} | {caret}^")
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "{} error: {}", self.format, self.message)
        } else {
            write!(
                f,
                "{} error at line {}, column {}: {}",
                self.format, self.line, self.column, self.message
            )
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct Server {
        host: String,
        port: u16,
    }

    #[test]
    fn picks_format_from_extension() {
        assert_eq!(Format::from_path(Path::new("a/b.json")), Some(Format::Json));
        assert_eq!(Format::from_path(Path::new("b.toml")), Some(Format::Toml));
        assert_eq!(Format::from_path(Path::new("b.yaml")), None);
        assert_eq!(Format::from_path(Path::new("README")), None);
    }

    #[test]
    fn json_errors_keep_line_and_column_apart_from_the_message() {
        let text = "{\n  \"host\": \"a\",\n  \"port\": \"80\"\n}";
        let err = Format::Json.parse::<Server>(text).unwrap_err();
        assert_eq!((err.line, err.column), (3, 14));
        assert_eq!(err.message, "invalid type: string \"80\", expected u16");
        assert_eq!(
            err.snippet(text),
            "  |\n3 |   \"port\": \"80\"\n  |              ^"
        );
    }

    #[test]
    fn toml_spans_become_line_and_column() {
        let text = "host = \"a\"\nport = \"80\"\n";
        let err = Format::Toml.parse::<Server>(text).unwrap_err();
        assert_eq!((err.line, err.column), (2, 8));
        assert!(
            err.to_string()
                .starts_with("TOML error at line 2, column 8: invalid type")
        );
    }

    #[test]
    fn line_and_column_counts_characters_not_bytes() {
        // `é` is 2 bytes: byte 8 is the `x`, the 5th character on line 2.
        assert_eq!(line_and_column("ab\ncdé x", 8), (2, 5));
        assert_eq!(line_and_column("x", 0), (1, 1));
    }

    #[test]
    fn toml_cannot_write_a_bare_list() {
        let err = Format::Toml.write(&vec![1, 2, 3]).unwrap_err();
        assert!(err.contains("unsupported"), "{err}");
        assert!(Format::Json.write(&vec![1, 2, 3]).is_ok());
    }
}
