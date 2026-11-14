//! Reading a `Cargo.toml` file.
//!
//! This is a deliberately small reader for the part of TOML that `Cargo.toml`
//! files use: `[sections]`, `[[array.tables]]`, `key = value`, strings,
//! numbers, booleans, arrays (also over several lines), inline tables and
//! `key.workspace = true`. A real tool would use the `toml` crate.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// One value on the right of `key = value`.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// `"text"` or `'text'`.
    String(String),
    /// `3`, `-1`, `1_000`.
    Integer(i64),
    /// `true` or `false`.
    Boolean(bool),
    /// `["a", "b"]`.
    Array(Vec<Value>),
    /// `{ version = "1", optional = true }`, kept as its raw text.
    Table(String),
    /// `key.workspace = true` or `key = { workspace = true }`: the value
    /// comes from the workspace's `[workspace.package]` table.
    Inherited,
}

impl Value {
    /// The text, if this is a [`Value::String`].
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// The items, if this is a [`Value::Array`].
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(items) => Some(items),
            _ => None,
        }
    }
}

/// Why a manifest could not be read, and on which line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// The line the problem starts on, counting from 1 like an editor.
    pub line: usize,
    /// What is wrong.
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

/// A parsed `Cargo.toml`: section name, then key, then value.
///
/// Keys inside `[package]` live in the section called `"package"`. Array
/// tables such as `[[bin]]` get a number: `"bin[0]"`, `"bin[1]"`, ...
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Manifest {
    sections: BTreeMap<String, BTreeMap<String, Value>>,
    inherited: BTreeSet<String>,
}

impl Manifest {
    /// Reads the text of a `Cargo.toml` file.
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] with the line number when the text is not
    /// valid for this small reader: a line without `=`, an unclosed string,
    /// array or section header, a key given twice, or a value it does not
    /// understand.
    ///
    /// # Examples
    ///
    /// ```
    /// use cargo_and_crates_io::{Manifest, Value};
    ///
    /// let manifest = Manifest::parse("[package]\nname = \"paint\"\nversion.workspace = true\n")?;
    /// assert_eq!(manifest.package_str("name"), Some("paint"));
    /// assert_eq!(manifest.get("package", "version"), Some(&Value::Inherited));
    /// # Ok::<(), cargo_and_crates_io::ParseError>(())
    /// ```
    pub fn parse(text: &str) -> Result<Manifest, ParseError> {
        let mut manifest = Manifest::default();
        let mut section = String::new(); // keys before any header: the root table
        let mut array_tables: BTreeMap<String, usize> = BTreeMap::new();
        let mut lines = text.lines().enumerate();

        while let Some((index, raw)) = lines.next() {
            let line_number = index + 1;
            let error = |message: &str| ParseError {
                line: line_number,
                message: message.to_string(),
            };
            let line = strip_comment(raw).trim();
            if line.is_empty() {
                continue;
            }

            if let Some(name) = line.strip_prefix("[[") {
                let name = name
                    .strip_suffix("]]")
                    .ok_or_else(|| error("unclosed [[array table]] header"))?
                    .trim();
                let count = array_tables.entry(name.to_string()).or_insert(0);
                section = format!("{name}[{count}]");
                *count += 1;
                manifest.sections.entry(section.clone()).or_default();
                continue;
            }
            if let Some(name) = line.strip_prefix('[') {
                let name = name
                    .strip_suffix(']')
                    .ok_or_else(|| error("unclosed [section] header"))?
                    .trim();
                if manifest.sections.contains_key(name) {
                    return Err(error(&format!("section [{name}] appears twice")));
                }
                section = name.to_string();
                manifest.sections.entry(section.clone()).or_default();
                continue;
            }

            let (key, value_text) = line
                .split_once('=')
                .ok_or_else(|| error("expected `key = value`"))?;
            let mut key = key.trim().to_string();

            // An array may continue on the next lines: keep reading until
            // every `[` and `{` is closed.
            let mut value_text = value_text.trim().to_string();
            while !is_balanced(&value_text) {
                let Some((_, next)) = lines.next() else {
                    return Err(error("this value is never closed"));
                };
                value_text.push(' ');
                value_text.push_str(strip_comment(next).trim());
            }

            if value_text.starts_with("\"\"\"") || value_text.starts_with("'''") {
                return Err(error(
                    "multi-line strings are not supported by this small reader",
                ));
            }
            let (mut value, rest) = parse_value(&value_text).map_err(|m| error(&m))?;
            if !rest.trim().is_empty() {
                return Err(error(&format!(
                    "unexpected text after the value: `{}`",
                    rest.trim()
                )));
            }

            // `version.workspace = true` means "take version from the workspace".
            if let Some(base) = key.strip_suffix(".workspace") {
                if value == Value::Boolean(true) {
                    key = base.to_string();
                    value = Value::Inherited;
                }
            }

            let table = manifest.sections.entry(section.clone()).or_default();
            if table.insert(key.clone(), value).is_some() {
                return Err(error(&format!("key `{key}` appears twice in [{section}]")));
            }
        }
        Ok(manifest)
    }

    /// The value of `key` in `[section]`, if there is one.
    pub fn get(&self, section: &str, key: &str) -> Option<&Value> {
        self.sections.get(section)?.get(key)
    }

    /// A string from the `[package]` table, such as `"name"` or `"license"`.
    pub fn package_str(&self, key: &str) -> Option<&str> {
        self.get("package", key)?.as_str()
    }

    /// Whether the file has a `[section]` with this name.
    pub fn has_section(&self, section: &str) -> bool {
        self.sections.contains_key(section)
    }

    /// Whether this is a workspace root: it has a `[workspace]` table.
    pub fn is_workspace(&self) -> bool {
        self.has_section("workspace")
    }

    /// The `members` listed in `[workspace]`, in the order written.
    pub fn workspace_members(&self) -> Vec<&str> {
        self.get("workspace", "members")
            .and_then(Value::as_array)
            .unwrap_or_default()
            .iter()
            .filter_map(Value::as_str)
            .collect()
    }

    /// Replaces every inherited `[package]` value (`key.workspace = true`)
    /// with the value from the workspace's `[workspace.package]` table.
    ///
    /// Afterwards [`Manifest::inherited_keys`] lists which keys came from
    /// the workspace. Keys the workspace does not define stay
    /// [`Value::Inherited`].
    pub fn inherit_from(&mut self, workspace: &Manifest) {
        let Some(package) = self.sections.get_mut("package") else {
            return;
        };
        for (key, value) in package.iter_mut() {
            if *value == Value::Inherited {
                if let Some(shared) = workspace.get("workspace.package", key) {
                    *value = shared.clone();
                    self.inherited.insert(key.clone());
                }
            }
        }
    }

    /// The `[package]` keys filled in by [`Manifest::inherit_from`].
    pub fn inherited_keys(&self) -> impl Iterator<Item = &str> {
        self.inherited.iter().map(String::as_str)
    }
}

/// Cuts a `# comment` off a line, but not a `#` inside a string.
fn strip_comment(line: &str) -> &str {
    let mut quote = None;
    let mut escaped = false;
    for (i, c) in line.char_indices() {
        match (quote, c) {
            (Some('"'), '\\') if !escaped => {
                escaped = true;
                continue;
            }
            (Some(q), c) if c == q && !escaped => quote = None,
            (None, '"' | '\'') => quote = Some(c),
            (None, '#') => return &line[..i],
            _ => {}
        }
        escaped = false;
    }
    line
}

/// True when every `[` and `{` outside strings has been closed.
fn is_balanced(text: &str) -> bool {
    let mut depth = 0i32;
    let mut quote = None;
    let mut escaped = false;
    for c in text.chars() {
        match (quote, c) {
            (Some('"'), '\\') if !escaped => {
                escaped = true;
                continue;
            }
            (Some(q), c) if c == q && !escaped => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '[' | '{') => depth += 1,
            (None, ']' | '}') => depth -= 1,
            _ => {}
        }
        escaped = false;
    }
    depth <= 0 && quote.is_none()
}

/// Reads one value from the start of `text`; returns it and the rest.
fn parse_value(text: &str) -> Result<(Value, &str), String> {
    let text = text.trim_start();
    let mut chars = text.char_indices();
    match chars.next() {
        None => Err("a value is missing after `=`".to_string()),
        Some((_, '"')) => {
            let mut out = String::new();
            let mut escaped = false;
            for (i, c) in chars {
                match (escaped, c) {
                    (false, '\\') => escaped = true,
                    (false, '"') => return Ok((Value::String(out), &text[i + 1..])),
                    (true, 'n') => {
                        out.push('\n');
                        escaped = false;
                    }
                    (true, 't') => {
                        out.push('\t');
                        escaped = false;
                    }
                    (_, c) => {
                        out.push(c);
                        escaped = false;
                    }
                }
            }
            Err("this string is never closed".to_string())
        }
        Some((_, '\'')) => {
            let end = text[1..].find('\'').ok_or("this string is never closed")?;
            Ok((Value::String(text[1..=end].to_string()), &text[end + 2..]))
        }
        Some((_, '[')) => {
            let mut items = Vec::new();
            let mut rest = text[1..].trim_start();
            loop {
                if let Some(after) = rest.strip_prefix(']') {
                    return Ok((Value::Array(items), after));
                }
                let (item, after) = parse_value(rest)?;
                items.push(item);
                rest = after.trim_start();
                if let Some(after) = rest.strip_prefix(',') {
                    rest = after.trim_start();
                } else if !rest.starts_with(']') {
                    return Err("expected `,` or `]` in an array".to_string());
                }
            }
        }
        Some((_, '{')) => {
            // Keep inline tables as raw text: find the matching `}`.
            let mut end = None;
            for (i, _) in text.char_indices().skip(1) {
                if is_balanced(&text[..=i]) {
                    end = Some(i);
                    break;
                }
            }
            let end = end.ok_or("this inline table is never closed")?;
            let raw = &text[..=end];
            let compact: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
            let value = if compact == "{workspace=true}" {
                Value::Inherited
            } else {
                Value::Table(raw.to_string())
            };
            Ok((value, &text[end + 1..]))
        }
        Some(_) => {
            let end = text
                .find(|c: char| c == ',' || c == ']' || c == '}' || c.is_whitespace())
                .unwrap_or(text.len());
            let (word, rest) = text.split_at(end);
            let value = match word {
                "true" => Value::Boolean(true),
                "false" => Value::Boolean(false),
                _ => word
                    .replace('_', "")
                    .parse()
                    .map(Value::Integer)
                    .map_err(|_| format!("`{word}` is not a value this reader understands"))?,
            };
            Ok((value, rest))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
# A comment on its own line
[package]
name = "paint"            # a comment after a value
version.workspace = true
edition = { workspace = true }
description = "Mixes colours # not a comment"
keywords = [
    "colour",   # comments inside arrays are fine
    "paint",
]
publish = false

[dependencies]
serde = { version = "1", features = ["derive"], optional = true }

[profile.dev]
opt-level = 1

[[bin]]
name = "one"

[[bin]]
name = "two"
"#;

    #[test]
    fn reads_strings_numbers_booleans_and_arrays() {
        let m = Manifest::parse(SAMPLE).unwrap();
        assert_eq!(m.package_str("name"), Some("paint"));
        assert_eq!(
            m.package_str("description"),
            Some("Mixes colours # not a comment")
        );
        assert_eq!(m.get("package", "publish"), Some(&Value::Boolean(false)));
        assert_eq!(m.get("profile.dev", "opt-level"), Some(&Value::Integer(1)));
        let keywords = m.get("package", "keywords").unwrap().as_array().unwrap();
        assert_eq!(keywords.len(), 2);
    }

    #[test]
    fn both_inheritance_spellings_are_understood() {
        let m = Manifest::parse(SAMPLE).unwrap();
        assert_eq!(m.get("package", "version"), Some(&Value::Inherited));
        assert_eq!(m.get("package", "edition"), Some(&Value::Inherited));
    }

    #[test]
    fn inline_tables_and_array_tables() {
        let m = Manifest::parse(SAMPLE).unwrap();
        assert!(matches!(
            m.get("dependencies", "serde"),
            Some(Value::Table(_))
        ));
        assert_eq!(m.get("bin[0]", "name").and_then(Value::as_str), Some("one"));
        assert_eq!(m.get("bin[1]", "name").and_then(Value::as_str), Some("two"));
    }

    #[test]
    fn inherit_from_fills_in_workspace_values() {
        let root = Manifest::parse(
            "[workspace]\nmembers = [\"paint\"]\n[workspace.package]\nversion = \"0.3.0\"\n",
        )
        .unwrap();
        let mut m = Manifest::parse(SAMPLE).unwrap();
        m.inherit_from(&root);
        assert_eq!(m.package_str("version"), Some("0.3.0"));
        // edition is not in [workspace.package], so it stays inherited
        assert_eq!(m.get("package", "edition"), Some(&Value::Inherited));
        assert_eq!(m.inherited_keys().collect::<Vec<_>>(), ["version"]);
        assert_eq!(root.workspace_members(), ["paint"]);
    }

    #[test]
    fn errors_carry_line_numbers() {
        let err = Manifest::parse("[package]\nname \"paint\"\n").unwrap_err();
        assert_eq!(err.line, 2);
        assert_eq!(err.to_string(), "line 2: expected `key = value`");

        let err = Manifest::parse("[package]\nname = \"paint\nversion = \"1\"").unwrap_err();
        assert_eq!(err.line, 2);

        let err = Manifest::parse("[package]\nname = \"a\"\nname = \"b\"\n").unwrap_err();
        assert!(err.message.contains("twice"), "{err}");
    }

    #[test]
    fn an_unclosed_array_is_an_error() {
        let err = Manifest::parse("members = [\n  \"a\",\n").unwrap_err();
        assert_eq!(err.line, 1);
    }
}
