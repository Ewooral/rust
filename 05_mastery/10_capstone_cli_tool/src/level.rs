//! Log levels: how serious a log line is.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// How serious a log line is, from least to most severe.
///
/// The variants are declared in order, so the derived `Ord` compares by
/// severity: `Level::Warn < Level::Error`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    /// Very detailed tracing.
    Trace,
    /// Information for developers.
    Debug,
    /// Normal events.
    Info,
    /// Something unexpected that the program survived.
    Warn,
    /// Something failed.
    Error,
    /// Something failed so badly that the program stopped.
    Fatal,
}

impl Level {
    /// Every level, least severe first.
    pub const ALL: [Level; 6] = [
        Level::Trace,
        Level::Debug,
        Level::Info,
        Level::Warn,
        Level::Error,
        Level::Fatal,
    ];

    /// The lowercase name: `"warn"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Level::Trace => "trace",
            Level::Debug => "debug",
            Level::Info => "info",
            Level::Warn => "warn",
            Level::Error => "error",
            Level::Fatal => "fatal",
        }
    }

    /// A position from 0 (trace) to 5 (fatal), for indexing arrays.
    pub fn index(self) -> usize {
        self as usize
    }
}

impl fmt::Display for Level {
    /// Uses the formatter's padding, so `{:<5}` lines up a table.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.as_str())
    }
}

/// The error for text that isn't a level name.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unknown level {0:?} (expected trace, debug, info, warn, error or fatal)")]
pub struct ParseLevelError(pub String);

impl FromStr for Level {
    type Err = ParseLevelError;

    /// Accepts the names real logs use, in any case: `WARN`, `warning`,
    /// `Err`, `CRITICAL`...
    ///
    /// ```
    /// use capstone_cli_tool::Level;
    /// assert_eq!("WARNING".parse(), Ok(Level::Warn));
    /// assert!("loud".parse::<Level>().is_err());
    /// ```
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text.to_ascii_lowercase().as_str() {
            "trace" => Ok(Level::Trace),
            "debug" => Ok(Level::Debug),
            "info" => Ok(Level::Info),
            "warn" | "warning" => Ok(Level::Warn),
            "error" | "err" => Ok(Level::Error),
            "fatal" | "critical" | "crit" => Ok(Level::Fatal),
            _ => Err(ParseLevelError(text.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_spellings_in_any_case() {
        assert_eq!("info".parse(), Ok(Level::Info));
        assert_eq!("WARNING".parse(), Ok(Level::Warn));
        assert_eq!("Err".parse(), Ok(Level::Error));
        assert_eq!("CRITICAL".parse(), Ok(Level::Fatal));
        assert_eq!("loud".parse::<Level>(), Err(ParseLevelError("loud".into())));
    }

    #[test]
    fn ordered_by_severity() {
        assert!(Level::Trace < Level::Debug);
        assert!(Level::Warn < Level::Error);
        assert_eq!(Level::ALL.iter().max(), Some(&Level::Fatal));
        assert_eq!(Level::Error.index(), 4);
    }

    #[test]
    fn display_respects_padding() {
        assert_eq!(format!("[{:<5}]", Level::Warn), "[warn ]");
        assert_eq!(format!("[{:>6}]", Level::Info), "[  info]");
    }

    #[test]
    fn serde_uses_lowercase_names() {
        assert_eq!(serde_json::to_string(&Level::Fatal).unwrap(), "\"fatal\"");
        let level: Level = serde_json::from_str("\"debug\"").unwrap();
        assert_eq!(level, Level::Debug);
    }
}
