//! The `Settings` trait that `#[derive(Settings)]` implements, and its errors.

use std::collections::BTreeMap;
use std::fmt;

/// What the derive knows about one field, for help text and key checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldInfo {
    /// The key in the settings text (the field name, or its `rename`).
    pub key: &'static str,
    /// The field's type, as written: `u16`, `Option<String>`.
    pub type_name: &'static str,
    /// The `#[setting(default = "…")]`, if any.
    pub default: Option<&'static str>,
    /// The field's `///` doc comment, on one line.
    pub doc: &'static str,
    /// `true` if loading fails without this key.
    pub required: bool,
}

/// Why settings couldn't be loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsError {
    /// A line that isn't `key = value`, a comment or blank.
    BadLine {
        /// The line number, from 1.
        line: usize,
        /// The line as written.
        text: String,
    },
    /// A key no field reads: usually a typo.
    UnknownKey {
        /// The key as written.
        key: String,
        /// A key that looks similar, if there is one.
        suggestion: Option<&'static str>,
    },
    /// A required key is missing.
    Missing {
        /// The key.
        key: &'static str,
    },
    /// The value didn't parse as the field's type.
    Invalid {
        /// The key.
        key: &'static str,
        /// The value as written.
        value: String,
        /// The `FromStr` error message.
        reason: String,
    },
    /// A rule checked after loading (see the `ensure!` macro).
    Rule(String),
}

impl fmt::Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingsError::BadLine { line, text } => {
                write!(f, "line {line}: expected `key = value`, found {text:?}")
            }
            SettingsError::UnknownKey { key, suggestion } => {
                write!(f, "unknown key `{key}`")?;
                match suggestion {
                    Some(similar) => write!(f, " (did you mean `{similar}`?)"),
                    None => Ok(()),
                }
            }
            SettingsError::Missing { key } => write!(f, "missing required key `{key}`"),
            SettingsError::Invalid { key, value, reason } => {
                write!(f, "invalid value {value:?} for `{key}`: {reason}")
            }
            SettingsError::Rule(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for SettingsError {}

/// A type that can be loaded from `key = value` settings.
///
/// Don't implement it by hand: `#[derive(Settings)]` writes `from_map` and
/// `fields`, and the other methods come for free.
pub trait Settings: Sized {
    /// Builds the value from keys and values that have already been read.
    ///
    /// # Errors
    ///
    /// [`SettingsError::Missing`] or [`SettingsError::Invalid`].
    fn from_map(map: &BTreeMap<String, String>) -> Result<Self, SettingsError>;

    /// One entry per field, in the order they are written.
    fn fields() -> &'static [FieldInfo];

    /// Like [`Settings::from_map`], but first rejects keys no field reads.
    ///
    /// # Errors
    ///
    /// [`SettingsError::UnknownKey`], then as for `from_map`.
    fn from_map_checked(map: &BTreeMap<String, String>) -> Result<Self, SettingsError> {
        for key in map.keys() {
            if !Self::fields().iter().any(|field| field.key == key) {
                return Err(SettingsError::UnknownKey {
                    key: key.clone(),
                    suggestion: suggest(key, Self::fields()),
                });
            }
        }
        Self::from_map(map)
    }

    /// Reads settings text: `key = value` lines, `#` comments, blank lines.
    ///
    /// # Errors
    ///
    /// [`SettingsError::BadLine`], then as for [`Settings::from_map_checked`].
    fn from_text(text: &str) -> Result<Self, SettingsError> {
        Self::from_map_checked(&parse_lines(text)?)
    }

    /// A help screen built from the fields' doc comments.
    fn help() -> String {
        let width = Self::fields()
            .iter()
            .map(|f| f.key.len())
            .max()
            .unwrap_or(0);
        let mut text = String::new();
        for field in Self::fields() {
            let note = match (field.default, field.required) {
                (Some(default), _) => format!(" [default: {default}]"),
                (None, true) => String::from(" [required]"),
                (None, false) => String::new(),
            };
            text.push_str(&format!(
                "  {:width$}  {}{note}\n  {:width$}  ({})\n",
                field.key, field.doc, "", field.type_name
            ));
        }
        text
    }
}

/// Splits settings text into keys and values. Later lines win.
///
/// # Errors
///
/// [`SettingsError::BadLine`] for a line with no `=`, or with nothing before it.
pub fn parse_lines(text: &str) -> Result<BTreeMap<String, String>, SettingsError> {
    let mut map = BTreeMap::new();
    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match line.split_once('=') {
            Some((key, value)) if !key.trim().is_empty() => {
                map.insert(key.trim().to_string(), value.trim().to_string());
            }
            _ => {
                return Err(SettingsError::BadLine {
                    line: index + 1,
                    text: line.to_string(),
                });
            }
        }
    }
    Ok(map)
}

/// A field key that matches `key` once case, `-` and `_` are ignored.
fn suggest(key: &str, fields: &[FieldInfo]) -> Option<&'static str> {
    let simplify = |s: &str| s.to_lowercase().replace(['-', '_'], "");
    fields
        .iter()
        .find(|field| simplify(field.key) == simplify(key))
        .map(|field| field.key)
}
