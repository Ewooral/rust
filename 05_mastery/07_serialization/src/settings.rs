//! The current settings format: version 3.
//!
//! Every attribute here is one from the lesson, used for a real reason:
//!
//! | Attribute | Why |
//! |---|---|
//! | `rename_all = "kebab-case"` | TOML files usually say `keep-versions`, Rust says `keep_versions` |
//! | `alias = "keep_versions"` | people still type the snake_case spelling by hand |
//! | `deny_unknown_fields` | a typo such as `prot = 8443` is an error, not silently ignored |
//! | `default`, `default = "..."` | optional settings get a sensible value |
//! | `skip_serializing_if` | don't write empty lists and `false` flags back out |
//! | `with = "crate::duration"` | `timeout = "30s"` instead of `{secs, nanos}` |
//! | `tag = "run"` | `Schedule` is an internally tagged enum: `run = "daily"` |
//! | `try_from`, `into` | `Port` and `TimeOfDay` check their values while parsing |

use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// The version this program writes. Older files are migrated up to it.
pub const CURRENT_VERSION: u32 = 3;

/// A backup tool's settings, as stored in `settings.toml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Settings {
    /// Always [`CURRENT_VERSION`] for this struct.
    pub version: u32,
    /// A name for this backup set, shown in the app.
    pub name: String,
    /// How many old copies of each file to keep.
    #[serde(default = "default_keep_versions", alias = "keep_versions")]
    pub keep_versions: u8,
    /// Where to send the backups.
    pub server: Server,
    /// When to run. Missing means "only when I press the button".
    #[serde(default)]
    pub schedule: Schedule,
    /// What to back up. Written as `[[folders]]` tables in TOML.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub folders: Vec<Folder>,
}

fn default_keep_versions() -> u8 {
    5
}

/// The backup server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
    /// A host name, such as `backup.example.com`.
    pub host: String,
    /// Never 0: [`Port`] refuses it while parsing.
    pub port: Port,
    /// How long to wait for the server, as `"30s"`, `"2m"`, ...
    #[serde(with = "crate::duration", default = "default_timeout")]
    pub timeout: Duration,
}

fn default_timeout() -> Duration {
    Duration::from_secs(30)
}

/// A TCP port that is not 0.
///
/// `#[serde(try_from = "u16", into = "u16")]` makes Serde read a plain
/// `u16` and then call `Port::try_from`, so a bad value fails *during*
/// parsing, with the file's line and column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct Port(u16);

impl Port {
    /// The number inside.
    pub fn get(self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for Port {
    type Error = String;

    fn try_from(number: u16) -> Result<Port, String> {
        if number == 0 {
            Err("port 0 means \"any free port\", which a server can't listen on".to_string())
        } else {
            Ok(Port(number))
        }
    }
}

impl From<Port> for u16 {
    fn from(port: Port) -> u16 {
        port.0
    }
}

/// When the backup runs.
///
/// Internally tagged: the variant's name is a field *inside* the table.
///
/// ```toml
/// [schedule]
/// run = "weekly"
/// on = "sunday"
/// at = "03:00"
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(tag = "run", rename_all = "lowercase", deny_unknown_fields)]
pub enum Schedule {
    /// Only when the user asks.
    #[default]
    Manually,
    /// Every hour, at this minute past the hour.
    Hourly {
        /// 0 to 59.
        minute: u8,
    },
    /// Every day at a time.
    Daily {
        /// The time of day.
        at: TimeOfDay,
    },
    /// Once a week.
    Weekly {
        /// The day of the week.
        on: Weekday,
        /// The time of day.
        at: TimeOfDay,
    },
}

/// A day of the week, written in lowercase. `alias` also accepts `"mon"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(missing_docs)]
pub enum Weekday {
    #[serde(alias = "mon")]
    Monday,
    #[serde(alias = "tue")]
    Tuesday,
    #[serde(alias = "wed")]
    Wednesday,
    #[serde(alias = "thu")]
    Thursday,
    #[serde(alias = "fri")]
    Friday,
    #[serde(alias = "sat")]
    Saturday,
    #[serde(alias = "sun")]
    Sunday,
}

/// A time such as `"02:30"`, checked while parsing.
///
/// In the file it is a string; in Rust it is two numbers. `try_from` and
/// `into` convert between the two, so nobody can build an invalid one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct TimeOfDay {
    hour: u8,
    minute: u8,
}

impl TimeOfDay {
    /// Builds a time, or explains why it isn't one.
    ///
    /// # Errors
    ///
    /// When the hour is over 23 or the minute over 59.
    pub fn new(hour: u8, minute: u8) -> Result<TimeOfDay, String> {
        if hour > 23 || minute > 59 {
            Err(format!("{hour:02}:{minute:02} is not a time of day"))
        } else {
            Ok(TimeOfDay { hour, minute })
        }
    }
}

impl TryFrom<String> for TimeOfDay {
    type Error = String;

    fn try_from(text: String) -> Result<TimeOfDay, String> {
        text.parse()
    }
}

impl std::str::FromStr for TimeOfDay {
    type Err = String;

    fn from_str(text: &str) -> Result<TimeOfDay, String> {
        let bad = || format!("expected a time like \"02:30\", found {text:?}");
        let (hour, minute) = text.split_once(':').ok_or_else(bad)?;
        if minute.len() != 2 {
            return Err(bad());
        }
        let hour: u8 = hour.parse().map_err(|_| bad())?;
        let minute: u8 = minute.parse().map_err(|_| bad())?;
        TimeOfDay::new(hour, minute)
    }
}

impl From<TimeOfDay> for String {
    fn from(time: TimeOfDay) -> String {
        time.to_string()
    }
}

impl fmt::Display for TimeOfDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.minute)
    }
}

/// One folder to back up.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Folder {
    /// The folder's path.
    pub path: String,
    /// File patterns such as `"*.jpg"`. Empty means every file.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include: Vec<String>,
    /// Follow shortcuts and symbolic links. Only written when `true`.
    #[serde(default, skip_serializing_if = "is_false")]
    pub follow_links: bool,
}

// `skip_serializing_if` wants a function that takes a reference.
fn is_false(value: &bool) -> bool {
    !*value
}

impl Folder {
    /// A folder with no patterns, not following links.
    pub fn new(path: impl Into<String>) -> Folder {
        Folder {
            path: path.into(),
            include: Vec::new(),
            follow_links: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOML: &str = r#"
version = 3
name = "Family photos"

[server]
host = "backup.example.com"
port = 8443

[schedule]
run = "weekly"
on = "sun"
at = "03:00"

[[folders]]
path = "C:/Users/ama/Pictures"
include = ["*.jpg", "*.png"]

[[folders]]
path = "D:/Camera"
follow-links = true
"#;

    #[test]
    fn reads_toml_with_defaults_and_aliases() {
        let settings: Settings = toml::from_str(TOML).unwrap();
        assert_eq!(settings.keep_versions, 5, "default");
        assert_eq!(settings.server.timeout, Duration::from_secs(30), "default");
        assert_eq!(settings.server.port.get(), 8443);
        assert_eq!(
            settings.schedule,
            Schedule::Weekly {
                on: Weekday::Sunday,
                at: TimeOfDay::new(3, 0).unwrap()
            }
        );
        assert_eq!(settings.folders.len(), 2);
        assert!(settings.folders[1].follow_links);
    }

    #[test]
    fn round_trips_through_toml_and_json() {
        let settings: Settings = toml::from_str(TOML).unwrap();
        let again: Settings = toml::from_str(&toml::to_string(&settings).unwrap()).unwrap();
        assert_eq!(again, settings);
        let json = serde_json::to_string_pretty(&settings).unwrap();
        assert_eq!(serde_json::from_str::<Settings>(&json).unwrap(), settings);
    }

    #[test]
    fn writes_only_what_matters() {
        let settings: Settings = toml::from_str(TOML).unwrap();
        let written = toml::to_string(&settings).unwrap();
        assert!(written.contains("keep-versions = 5"));
        assert!(written.contains("timeout = \"30s\""));
        assert!(
            written.contains("on = \"sunday\""),
            "aliases are read, never written"
        );
        assert_eq!(
            written.matches("follow-links").count(),
            1,
            "false is skipped"
        );
    }

    #[test]
    fn snake_case_alias_is_accepted() {
        let text = TOML.replace("name = ", "keep_versions = 9\nname = ");
        let settings: Settings = toml::from_str(&text).unwrap();
        assert_eq!(settings.keep_versions, 9);
    }

    #[test]
    fn typos_are_errors() {
        let text = TOML.replace("port = 8443", "prot = 8443");
        let err = toml::from_str::<Settings>(&text).unwrap_err();
        assert!(err.message().contains("unknown field `prot`"), "{err}");
    }

    #[test]
    fn port_zero_and_bad_times_fail_while_parsing() {
        let err = toml::from_str::<Settings>(&TOML.replace("8443", "0")).unwrap_err();
        assert!(err.message().contains("port 0"), "{err}");
        let err = toml::from_str::<Settings>(&TOML.replace("03:00", "25:00")).unwrap_err();
        assert!(
            err.message().contains("25:00 is not a time of day"),
            "{err}"
        );
        assert!("7:5".parse::<TimeOfDay>().is_err());
        assert_eq!("7:05".parse::<TimeOfDay>().unwrap().to_string(), "07:05");
    }

    #[test]
    fn missing_schedule_means_manually() {
        let start = TOML.find("[schedule]").unwrap();
        let end = TOML.find("[[folders]]").unwrap();
        let text = format!("{}{}", &TOML[..start], &TOML[end..]);
        let settings: Settings = toml::from_str(&text).unwrap();
        assert_eq!(settings.schedule, Schedule::Manually);
    }
}
