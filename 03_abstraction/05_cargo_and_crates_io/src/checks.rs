//! The checks: what crates.io needs, and what makes a crate easy to find.

use crate::licenses::unknown_licenses;
use crate::manifest::{Manifest, Value};

/// How serious a [`Finding`] is. `Error` sorts first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[cfg_attr(feature = "json", serde(rename_all = "lowercase"))]
pub enum Severity {
    /// crates.io would refuse the upload.
    Error,
    /// The upload works, but the crate's page would be worse for it.
    Warning,
    /// Good to know; nothing to fix.
    Info,
}

/// One thing the checker noticed about one field.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(serde::Serialize))]
pub struct Finding {
    /// How serious it is.
    pub severity: Severity,
    /// The `Cargo.toml` field it is about, such as `"license"`.
    pub field: &'static str,
    /// What is wrong, or what to know, in plain English.
    pub message: String,
}

impl Finding {
    /// Creates a finding.
    ///
    /// # Panics
    ///
    /// Panics if `message` is empty: a finding must always say something.
    /// This is a bug in the caller, not bad input, so it panics instead of
    /// returning a `Result`.
    pub fn new(severity: Severity, field: &'static str, message: impl Into<String>) -> Finding {
        let message = message.into();
        assert!(!message.is_empty(), "a finding needs a message");
        Finding {
            severity,
            field,
            message,
        }
    }
}

/// The most keywords crates.io accepts.
pub const MAX_KEYWORDS: usize = 5;

/// Checks a manifest the way `cargo publish` and crates.io would, and
/// returns everything it noticed, errors first.
///
/// Call [`Manifest::inherit_from`] first, so that fields inherited from the
/// workspace are checked with their real values.
///
/// # Examples
///
/// ```
/// use cargo_and_crates_io::{check, Manifest, Severity};
///
/// let manifest = Manifest::parse("[package]\nname = \"paint\"\nversion = \"0.1.0\"\n")?;
/// let findings = check(&manifest);
/// // No description and no licence: crates.io would say no.
/// assert!(findings.iter().any(|f| f.severity == Severity::Error && f.field == "license"));
/// # Ok::<(), cargo_and_crates_io::ParseError>(())
/// ```
pub fn check(manifest: &Manifest) -> Vec<Finding> {
    use Severity::{Error, Info, Warning};
    let mut found = Vec::new();
    let mut add =
        |severity, field, message: String| found.push(Finding::new(severity, field, message));

    if !manifest.has_section("package") {
        add(
            Error,
            "package",
            "no [package] table: this is a virtual workspace manifest, not a crate".into(),
        );
        return found;
    }

    match manifest.get("package", "name").and_then(Value::as_str) {
        None => add(Error, "name", "a crate needs a `name`".into()),
        Some(name) if !is_valid_crate_name(name) => add(
            Error,
            "name",
            format!("`{name}` is not a valid crate name (letters, digits, `-`, `_`; max 64)"),
        ),
        Some(_) => {}
    }

    match manifest.get("package", "version") {
        None => add(
            Error,
            "version",
            "a published crate needs a `version`".into(),
        ),
        Some(Value::Inherited) => add(
            Info,
            "version",
            "inherited from a workspace that was not found, so not checked".into(),
        ),
        Some(Value::String(v)) if !looks_like_semver(v) => add(
            Error,
            "version",
            format!("`{v}` is not a semantic version like 1.2.3"),
        ),
        Some(_) => {}
    }

    if manifest.package_str("description").is_none() {
        add(
            Error,
            "description",
            "crates.io requires a `description` (one sentence is enough)".into(),
        );
    }

    let license = manifest.package_str("license");
    let license_file = manifest.package_str("license-file");
    match (license, license_file) {
        (None, None) => add(
            Error,
            "license",
            "crates.io requires `license` (an SPDX expression) or `license-file`".into(),
        ),
        (Some(expression), _) => {
            if expression.contains('/') {
                add(
                    Warning,
                    "license",
                    format!(
                        "`{expression}` uses the old `/` spelling: write `{}`",
                        expression.replace('/', " OR ")
                    ),
                );
            }
            for id in unknown_licenses(expression) {
                add(
                    Warning,
                    "license",
                    format!("`{id}` is not a known SPDX licence identifier"),
                );
            }
        }
        (None, Some(_)) => {}
    }

    if manifest.package_str("repository").is_none() {
        add(
            Warning,
            "repository",
            "no `repository` link: people like to read the source before they depend on it".into(),
        );
    }

    if manifest.get("package", "edition").is_none() {
        add(
            Warning,
            "edition",
            "no `edition`, so Cargo assumes 2015: set it to \"2024\"".into(),
        );
    }

    match manifest
        .get("package", "keywords")
        .and_then(Value::as_array)
    {
        None => add(
            Info,
            "keywords",
            format!("add up to {MAX_KEYWORDS} `keywords` so people can find the crate"),
        ),
        Some(words) => {
            if words.len() > MAX_KEYWORDS {
                add(
                    Error,
                    "keywords",
                    format!(
                        "{} keywords: crates.io allows at most {MAX_KEYWORDS}",
                        words.len()
                    ),
                );
            }
            for word in words.iter().filter_map(Value::as_str) {
                if !is_valid_keyword(word) {
                    add(
                        Error,
                        "keywords",
                        format!(
                            "keyword `{word}`: use at most 20 ASCII letters, digits, `-` or `_`"
                        ),
                    );
                }
            }
        }
    }

    if manifest.get("package", "categories").is_none() {
        add(
            Info,
            "categories",
            "no `categories`: see https://crates.io/category_slugs".into(),
        );
    }

    if manifest.get("package", "publish") == Some(&Value::Boolean(false)) {
        add(
            Info,
            "publish",
            "`publish = false`: `cargo publish` will refuse, on purpose".into(),
        );
    }

    let inherited: Vec<&str> = manifest.inherited_keys().collect();
    if !inherited.is_empty() {
        add(
            Info,
            "workspace",
            format!("from [workspace.package]: {}", inherited.join(", ")),
        );
    }

    found.sort_by_key(|f| f.severity); // a stable sort keeps the field order
    found
}

/// True when none of the findings is an [`Severity::Error`].
pub fn passes(findings: &[Finding]) -> bool {
    findings.iter().all(|f| f.severity != Severity::Error)
}

/// crates.io names: ASCII letters, digits, `-` and `_`, starting with a
/// letter, at most 64 characters.
fn is_valid_crate_name(name: &str) -> bool {
    name.len() <= 64
        && name.starts_with(|c: char| c.is_ascii_alphabetic())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// `1.2.3`, optionally followed by `-pre.release` or `+build`.
fn looks_like_semver(version: &str) -> bool {
    let core = version.split(['-', '+']).next().unwrap_or_default();
    let parts: Vec<&str> = core.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

fn is_valid_keyword(word: &str) -> bool {
    !word.is_empty()
        && word.len() <= 20
        && word.starts_with(|c: char| c.is_ascii_alphanumeric())
        && word
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '+')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn findings(toml: &str) -> Vec<Finding> {
        check(&Manifest::parse(toml).unwrap())
    }

    fn has(findings: &[Finding], severity: Severity, field: &str) -> bool {
        findings
            .iter()
            .any(|f| f.severity == severity && f.field == field)
    }

    const COMPLETE: &str = r#"
[package]
name = "paint"
version = "1.0.0"
edition = "2024"
description = "Mixes colours"
license = "MIT OR Apache-2.0"
repository = "https://example.com/paint"
keywords = ["colour", "paint"]
categories = ["graphics"]
"#;

    #[test]
    fn a_complete_manifest_passes_with_nothing_to_say() {
        assert_eq!(findings(COMPLETE), []);
    }

    #[test]
    fn the_bare_minimum_is_not_enough_for_crates_io() {
        let f = findings("[package]\nname = \"paint\"\nversion = \"0.1.0\"\nedition = \"2024\"\n");
        assert!(has(&f, Severity::Error, "description"));
        assert!(has(&f, Severity::Error, "license"));
        assert!(has(&f, Severity::Warning, "repository"));
        assert!(!passes(&f));
        assert_eq!(f[0].severity, Severity::Error, "errors come first");
    }

    #[test]
    fn a_virtual_workspace_is_not_a_crate() {
        let f = findings("[workspace]\nmembers = [\"a\"]\n");
        assert_eq!(f.len(), 1);
        assert!(has(&f, Severity::Error, "package"));
    }

    #[test]
    fn licence_problems_are_warnings() {
        let f = findings(&COMPLETE.replace("MIT OR Apache-2.0", "MIT/Apache-2.0"));
        assert!(has(&f, Severity::Warning, "license"));
        let f = findings(&COMPLETE.replace("MIT OR Apache-2.0", "MIT OR Cheese-1.0"));
        assert!(f[0].message.contains("Cheese-1.0"));
        assert!(passes(&f), "a warning does not stop publishing");
    }

    #[test]
    fn keywords_follow_crates_io_rules() {
        let six = r#"keywords = ["a", "b", "c", "d", "e", "f"]"#;
        let f = findings(&COMPLETE.replace(r#"keywords = ["colour", "paint"]"#, six));
        assert!(has(&f, Severity::Error, "keywords"));
        let long = r#"keywords = ["a-keyword-that-is-far-too-long"]"#;
        let f = findings(&COMPLETE.replace(r#"keywords = ["colour", "paint"]"#, long));
        assert!(has(&f, Severity::Error, "keywords"));
    }

    #[test]
    fn bad_names_and_versions_are_errors() {
        let f = findings(&COMPLETE.replace("\"paint\"", "\"9 lives\""));
        assert!(has(&f, Severity::Error, "name"));
        let f = findings(&COMPLETE.replace("1.0.0", "1.0"));
        assert!(has(&f, Severity::Error, "version"));
        let f = findings(&COMPLETE.replace("1.0.0", "1.0.0-beta.1"));
        assert!(passes(&f), "pre-releases are fine");
    }

    #[test]
    fn publish_false_and_inheritance_are_reported_as_info() {
        let root = Manifest::parse("[workspace.package]\nversion = \"0.2.0\"\n").unwrap();
        let mut m = Manifest::parse(&COMPLETE.replace(
            "version = \"1.0.0\"",
            "version.workspace = true\npublish = false",
        ))
        .unwrap();
        m.inherit_from(&root);
        let f = check(&m);
        assert!(has(&f, Severity::Info, "publish"));
        assert!(has(&f, Severity::Info, "workspace"));
        assert!(passes(&f));
    }

    #[test]
    #[should_panic(expected = "a finding needs a message")]
    fn an_empty_message_is_a_bug() {
        Finding::new(Severity::Info, "name", "");
    }
}
