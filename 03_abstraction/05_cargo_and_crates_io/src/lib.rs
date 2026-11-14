//! # cargo preflight
//!
//! The mini project for Book chapter 14: a checker that reads a crate's
//! `Cargo.toml` and tells you whether crates.io would accept it, **before**
//! you run `cargo publish` and find out the hard way.
//!
//! It is written the way a crate on crates.io should be: every public item
//! has a doc comment, the examples in them are tested by `cargo test`, and
//! the most useful items are re-exported at the top with `pub use`, so you
//! can write `cargo_and_crates_io::check` instead of
//! `cargo_and_crates_io::checks::check`.
//!
//! See the documentation with
//! `cargo doc -p cargo_and_crates_io --no-deps --open`.
//!
//! # Examples
//!
//! ```
//! use cargo_and_crates_io::{Manifest, Severity, check};
//!
//! let manifest = Manifest::parse(
//!     r#"
//! [package]
//! name = "paint"
//! version = "0.1.0"
//! edition = "2024"
//! description = "Mixes colours"
//! license = "MIT"
//! "#,
//! )?;
//! let findings = check(&manifest);
//! assert!(findings.iter().all(|f| f.severity != Severity::Error));
//! # Ok::<(), cargo_and_crates_io::ParseError>(())
//! ```
//!
//! # Features
//!
//! - `unicode` (on by default): marks like ✓ and ✗. Turn it off with
//!   `--no-default-features` for plain `[ok]` and `[x]`.
//! - `json`: adds `report::render_json` and the `--json` option. It
//!   switches on the optional `serde` and `serde_json` dependencies. (No
//!   `[link]` here: without the feature the function does not exist, and
//!   rustdoc would warn about a broken link.)

pub mod checks;
pub mod cli;
pub mod licenses;
pub mod manifest;
pub mod report;
pub mod workspace;

// The public API, flattened: users need not learn our module layout.
pub use checks::{Finding, Severity, check, passes};
pub use licenses::{KNOWN_LICENSES, is_known_license};
pub use manifest::{Manifest, ParseError, Value};

#[cfg(test)]
mod tests {
    use super::*;

    /// The lesson's own Cargo.toml must pass its own checks.
    #[test]
    fn this_lessons_manifest_is_ready_apart_from_publish_false() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let text = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        let mut manifest = Manifest::parse(&text).unwrap();
        let (_, root) = workspace::find_workspace_root(&dir.join("Cargo.toml")).unwrap();
        manifest.inherit_from(&root);
        let findings = check(&manifest);
        assert!(passes(&findings), "{findings:#?}");
        assert!(findings.iter().any(|f| f.field == "publish"));
        assert_eq!(
            manifest.package_str("version"),
            Some(env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn a_re_export_is_the_same_item_under_a_shorter_name() {
        use std::any::type_name;
        assert_eq!(type_name::<Finding>(), type_name::<checks::Finding>());
        assert_eq!(
            type_name::<Finding>(),
            "cargo_and_crates_io::checks::Finding"
        );
    }
}
