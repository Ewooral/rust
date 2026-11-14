//! # Settings from macros
//!
//! The mini project for "Macros in depth": a small library for loading typed
//! settings from `key = value` text, where macros write most of the code.
//!
//! | Macro | Kind | Does |
//! |---|---|---|
//! | `#[derive(Settings)]` | derive (proc macro) | reads each field with `FromStr`, help text from `///` docs |
//! | `palette! { … }` | function-like (proc macro) | colour constants, checked at compile time |
//! | `#[memoize]` | attribute (proc macro) | remembers a function's results |
//! | [`settings_map!`] | `macro_rules!` | a `BTreeMap` literal |
//! | [`ensure!`] | `macro_rules!` | return an error unless a rule holds |
//!
//! The procedural macros live in their own crate, `macros_in_depth_derive`
//! (the `macros/` folder), and are re-exported here, next to the trait and
//! types their generated code uses.
//!
//! ```
//! use macros_in_depth::Settings;
//!
//! #[derive(Settings)]
//! struct Server {
//!     /// Port to listen on.
//!     #[setting(default = "8080")]
//!     port: u16,
//!     /// Log file, if any.
//!     log_file: Option<String>,
//! }
//!
//! let server = Server::from_text("log_file = server.log").unwrap();
//! assert_eq!(server.port, 8080);
//! assert_eq!(server.log_file.as_deref(), Some("server.log"));
//! ```

// The derive writes `::macros_in_depth::Settings`. In OTHER crates that path
// names this library. Inside this library's own tests it would not resolve,
// so give the crate its own name as an extern crate too. Procedural macros
// have no `$crate`; this is the usual work-around.
extern crate self as macros_in_depth;

mod color;
#[macro_use]
mod macros;
mod settings;

pub use color::Rgb;
// A derive macro and a trait may share a name: macros and types live in
// different namespaces. `use macros_in_depth::Settings;` imports both.
pub use macros_in_depth_derive::{Settings, memoize, palette};
pub use settings::{FieldInfo, Settings, SettingsError, parse_lines};

/// Helpers for macro-generated code only. Not part of the public API: they
/// may change in any release. (`serde` has the same kind of module.)
#[doc(hidden)]
pub mod __private {
    use std::collections::BTreeMap;
    use std::fmt::Display;
    use std::str::FromStr;

    use crate::SettingsError;

    /// Reads a key that must be there (or have a default).
    pub fn required<T>(
        map: &BTreeMap<String, String>,
        key: &'static str,
        default: Option<&'static str>,
    ) -> Result<T, SettingsError>
    where
        T: FromStr,
        T::Err: Display,
    {
        let text = match map.get(key) {
            Some(value) => value.as_str(),
            None => default.ok_or(SettingsError::Missing { key })?,
        };
        parse(key, text)
    }

    /// Reads a key that may be missing: an `Option<T>` field.
    pub fn optional<T>(
        map: &BTreeMap<String, String>,
        key: &'static str,
        _default: Option<&'static str>,
    ) -> Result<Option<T>, SettingsError>
    where
        T: FromStr,
        T::Err: Display,
    {
        map.get(key).map(|text| parse(key, text)).transpose()
    }

    fn parse<T>(key: &'static str, text: &str) -> Result<T, SettingsError>
    where
        T: FromStr,
        T::Err: Display,
    {
        text.parse()
            .map_err(|error: T::Err| SettingsError::Invalid {
                key,
                value: text.to_string(),
                reason: error.to_string(),
            })
    }

    /// Used by `settings_map!`.
    pub fn to_value(value: &impl Display) -> String {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    /// A test struct: one field of every kind the derive handles.
    #[derive(Debug, Settings)]
    struct Server {
        /// Host name.
        #[setting(default = "127.0.0.1")]
        host: String,
        /// Port to listen on.
        #[setting(default = "8080")]
        port: u16,
        /// Name shown in responses.
        name: String,
        /// Log file.
        #[setting(rename = "log-file")]
        log_file: Option<String>,
    }

    #[test]
    fn defaults_fill_the_gaps() {
        let server = Server::from_text("name = demo").unwrap();
        assert_eq!(server.host, "127.0.0.1");
        assert_eq!(server.port, 8080);
        assert_eq!(server.name, "demo");
        assert_eq!(server.log_file, None);
    }

    #[test]
    fn values_and_renamed_keys_are_read() {
        let text = "# a comment\n\nname = demo\nport = 9000\nlog-file = /tmp/s.log\n";
        let server = Server::from_text(text).unwrap();
        assert_eq!(server.port, 9000);
        assert_eq!(server.log_file.as_deref(), Some("/tmp/s.log"));
    }

    #[test]
    fn errors_say_what_went_wrong() {
        let missing = Server::from_text("port = 1").unwrap_err();
        assert_eq!(missing, SettingsError::Missing { key: "name" });

        let invalid = Server::from_text("name = x\nport = 99999").unwrap_err();
        assert_eq!(
            invalid.to_string(),
            "invalid value \"99999\" for `port`: number too large to fit in target type"
        );

        let typo = Server::from_text("name = x\nlog_file = a.log").unwrap_err();
        assert_eq!(
            typo.to_string(),
            "unknown key `log_file` (did you mean `log-file`?)"
        );

        let bad_line = Server::from_text("name = x\nport 80").unwrap_err();
        assert_eq!(
            bad_line,
            SettingsError::BadLine {
                line: 2,
                text: "port 80".into()
            }
        );
    }

    #[test]
    fn the_derive_records_docs_and_types() {
        let fields = Server::fields();
        assert_eq!(fields.len(), 4);
        assert_eq!(
            fields[1],
            FieldInfo {
                key: "port",
                type_name: "u16",
                default: Some("8080"),
                doc: "Port to listen on.",
                required: false,
            }
        );
        assert!(fields[2].required);
        assert_eq!(fields[3].type_name, "Option<String>");
        assert!(Server::help().contains("Name shown in responses. [required]"));
    }

    #[test]
    fn settings_map_builds_a_map() {
        let map = settings_map! { "name" => "demo", "port" => 7000 };
        let server = Server::from_map_checked(&map).unwrap();
        assert_eq!((server.name.as_str(), server.port), ("demo", 7000));
    }

    fn check(port: u16) -> Result<u16, SettingsError> {
        ensure!(port != 0, "port {} is not allowed", port);
        Ok(port)
    }

    #[test]
    fn ensure_returns_early() {
        assert_eq!(check(80), Ok(80));
        assert_eq!(
            check(0),
            Err(SettingsError::Rule("port 0 is not allowed".into()))
        );
    }

    palette! {
        mod test_colours {
            orange = "#f80",
            navy = "#000080",
        }
    }

    #[test]
    fn palette_makes_checked_constants() {
        assert_eq!(test_colours::ORANGE, Rgb::new(255, 136, 0));
        assert_eq!(test_colours::NAVY.to_string(), "#000080");
        assert_eq!(test_colours::ALL.len(), 2);
        assert_eq!(test_colours::ALL[0].0, "orange");
    }

    thread_local! {
        static REAL_CALLS: Cell<u32> = const { Cell::new(0) };
    }

    #[memoize]
    fn slow_square(n: u64) -> u64 {
        REAL_CALLS.with(|calls| calls.set(calls.get() + 1));
        n * n
    }

    #[memoize]
    fn fibonacci(n: u64) -> u64 {
        if n < 2 {
            return n; // an early `return` still ends up in the cache
        }
        fibonacci(n - 1) + fibonacci(n - 2)
    }

    #[test]
    fn memoize_runs_each_input_once() {
        assert_eq!(slow_square(12), 144);
        assert_eq!(slow_square(12), 144);
        assert_eq!(slow_square(5), 25);
        assert_eq!(REAL_CALLS.with(Cell::get), 2);
        // Without the cache this would take about 2^90 calls.
        assert_eq!(fibonacci(90), 2_880_067_194_370_816_120);
    }
}
