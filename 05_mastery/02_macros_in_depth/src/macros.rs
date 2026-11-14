//! The library's `macro_rules!` macros.
//!
//! `#[macro_export]` puts each one at the crate root, so users write
//! `macros_in_depth::settings_map!` (or `use macros_in_depth::settings_map;`),
//! whichever module it is defined in. Inside an exported macro, `$crate`
//! means "this library", however the user named or imported it.

/// A `BTreeMap<String, String>` of settings from `"key" => value` pairs.
///
/// Values can be anything that implements `Display`.
///
/// ```
/// let map = macros_in_depth::settings_map! {
///     "port" => 8080,
///     "host" => "example.org", // a trailing comma is fine
/// };
/// assert_eq!(map["port"], "8080");
/// assert_eq!(macros_in_depth::settings_map!().len(), 0);
/// ```
///
/// Keys must be literals:
///
/// ```compile_fail
/// let key = "port";
/// let map = macros_in_depth::settings_map! { key => 8080 };
/// ```
#[macro_export]
macro_rules! settings_map {
    () => {
        ::std::collections::BTreeMap::<::std::string::String, ::std::string::String>::new()
    };
    ($($key:literal => $value:expr),+ $(,)?) => {{
        let mut map = ::std::collections::BTreeMap::new();
        $(
            map.insert(
                ::std::string::String::from($key),
                $crate::__private::to_value(&$value),
            );
        )+
        map
    }};
}

/// Returns `Err(SettingsError::Rule(message))` from the current function
/// unless `condition` is true. The message is written like `format!`.
///
/// ```
/// use macros_in_depth::{SettingsError, ensure};
///
/// fn check_port(port: u16) -> Result<(), SettingsError> {
///     ensure!(port >= 1024, "port {port} is below 1024");
///     Ok(())
/// }
/// assert!(check_port(8080).is_ok());
/// assert_eq!(
///     check_port(80).unwrap_err().to_string(),
///     "port 80 is below 1024"
/// );
/// ```
#[macro_export]
macro_rules! ensure {
    ($condition:expr, $($message:tt)+) => {
        if !$condition {
            return ::core::result::Result::Err($crate::SettingsError::Rule(
                ::std::format!($($message)+),
            ));
        }
    };
}
