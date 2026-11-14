//! The mini project: a **settings loader** for a (pretend) backup tool.
//!
//! It reads a settings file in JSON or TOML, works out which *version* of
//! the format it is, upgrades old versions step by step, checks the result,
//! and can write it back out in the current format.
//!
//! | Module | Lesson topic |
//! |---|---|
//! | [`settings`] | derive, field attributes, enum representations, `try_from` |
//! | [`duration`] | `#[serde(with = ...)]` and a hand-written `Visitor` |
//! | [`format`] | one type, many formats; errors with line and column |
//! | [`legacy`], [`migrate`] | versioning: probe, parse, upgrade |
//! | [`validate`] | checks after parsing, reporting every problem |

pub mod duration;
pub mod format;
pub mod legacy;
pub mod migrate;
pub mod settings;
pub mod validate;

use std::fmt;

pub use format::{Format, ParseError};
pub use migrate::{MigrateError, Migrated};
pub use settings::{CURRENT_VERSION, Settings};
pub use validate::Problem;

/// Everything that can stop a settings file from loading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    /// The file couldn't be parsed or upgraded.
    Migrate(MigrateError),
    /// The file parsed, but the settings make no sense.
    Invalid(Vec<Problem>),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Migrate(err) => err.fmt(f),
            LoadError::Invalid(problems) => {
                write!(f, "{} problem(s):", problems.len())?;
                for problem in problems {
                    write!(f, "\n  - {problem}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for LoadError {}

impl From<MigrateError> for LoadError {
    fn from(err: MigrateError) -> LoadError {
        LoadError::Migrate(err)
    }
}

/// Parse, migrate and validate, in that order.
///
/// # Errors
///
/// See [`LoadError`].
///
/// # Examples
///
/// ```
/// use serialization::{Format, load};
///
/// let text = r#"
/// version = 3
/// name = "Photos"
/// [server]
/// host = "nas.local"
/// port = 9000
/// [[folders]]
/// path = "C:/Pictures"
/// "#;
/// let migrated = load(text, Format::Toml)?;
/// assert_eq!(migrated.settings.server.timeout.as_secs(), 30); // the default
/// # Ok::<(), serialization::LoadError>(())
/// ```
pub fn load(text: &str, format: Format) -> Result<Migrated, LoadError> {
    let migrated = migrate::read_any(text, format)?;
    let problems = validate::validate(&migrated.settings);
    if problems.is_empty() {
        Ok(migrated)
    } else {
        Err(LoadError::Invalid(problems))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_reports_problems_after_a_successful_parse() {
        let text = "version = 3\nname = \"\"\n[server]\nhost = \"h\"\nport = 1\n";
        let Err(LoadError::Invalid(problems)) = load(text, Format::Toml) else {
            panic!("expected validation problems");
        };
        assert_eq!(problems.len(), 2, "{problems:?}"); // empty name, no folders
    }

    #[test]
    fn load_passes_parse_errors_through() {
        let err = load("{", Format::Json).unwrap_err();
        assert!(matches!(
            err,
            LoadError::Migrate(MigrateError::Parse(ParseError { line: 1, .. }))
        ));
    }
}
