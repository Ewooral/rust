//! Reading any version, and upgrading it one step at a time.
//!
//! The recipe:
//!
//! 1. **Probe**: read *only* the `version` field, with a tiny struct that
//!    ignores everything else. A missing version means version 1.
//! 2. **Parse** the text again as that version's own struct.
//! 3. **Upgrade** step by step: v1 -> v2 -> v3. Each step is a plain Rust
//!    function from one struct to the next, so it is easy to test, and a
//!    new version only ever needs one new step.
//!
//! An internally tagged enum (`#[serde(tag = "version")]`) looks tempting,
//! but tags must be strings: `"version": 2` fails with "invalid type:
//! integer `2`, expected variant identifier".

use std::time::Duration;

use serde::Deserialize;

use crate::format::{Format, ParseError};
use crate::legacy::{SettingsV1, SettingsV2};
use crate::settings::{CURRENT_VERSION, Folder, Port, Schedule, Server, Settings, Weekday};

/// Only the version. Serde ignores unknown fields by default, so this
/// struct reads happily from a file of any version.
#[derive(Debug, Deserialize)]
struct Probe {
    version: Option<u32>,
}

/// The settings, upgraded to the current version, plus what was changed.
#[derive(Debug, Clone, PartialEq)]
pub struct Migrated {
    /// The settings, now at [`CURRENT_VERSION`].
    pub settings: Settings,
    /// The version the file was written in.
    pub from_version: u32,
    /// One line per change, to show the user.
    pub notes: Vec<String>,
}

/// Why a file couldn't be read or upgraded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrateError {
    /// The text didn't parse (as the probe or as its version's struct).
    Parse(ParseError),
    /// Written by a newer program than this one.
    TooNew(u32),
    /// A version number that never existed.
    UnknownVersion(u32),
    /// An old value that has no meaning in the new format.
    CannotUpgrade(String),
}

impl std::fmt::Display for MigrateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrateError::Parse(err) => err.fmt(f),
            MigrateError::TooNew(v) => write!(
                f,
                "version {v} was written by a newer program (this one reads up to {CURRENT_VERSION})"
            ),
            MigrateError::UnknownVersion(v) => write!(f, "there is no settings version {v}"),
            MigrateError::CannotUpgrade(why) => write!(f, "can't upgrade: {why}"),
        }
    }
}

impl std::error::Error for MigrateError {}

impl From<ParseError> for MigrateError {
    fn from(err: ParseError) -> MigrateError {
        MigrateError::Parse(err)
    }
}

/// Reads settings of any version and upgrades them to the current one.
///
/// # Errors
///
/// See [`MigrateError`].
///
/// # Examples
///
/// ```
/// use serialization::{Format, migrate::read_any};
///
/// let v1 = r#"{"name": "Photos", "server_url": "nas.local", "folders": ["C:/Pictures"]}"#;
/// let migrated = read_any(v1, Format::Json)?;
/// assert_eq!(migrated.from_version, 1);
/// assert_eq!(migrated.settings.version, 3);
/// assert_eq!(migrated.settings.server.port.get(), 443);
/// # Ok::<(), serialization::migrate::MigrateError>(())
/// ```
pub fn read_any(text: &str, format: Format) -> Result<Migrated, MigrateError> {
    let probe: Probe = format.parse(text)?;
    let from_version = probe.version.unwrap_or(1);
    let mut notes = Vec::new();
    let settings = match from_version {
        1 => {
            let v1: SettingsV1 = format.parse(text)?;
            let v2 = v1_to_v2(v1, &mut notes)?;
            v2_to_v3(v2, &mut notes)?
        }
        2 => v2_to_v3(format.parse(text)?, &mut notes)?,
        CURRENT_VERSION => format.parse(text)?,
        v if v > CURRENT_VERSION => return Err(MigrateError::TooNew(v)),
        v => return Err(MigrateError::UnknownVersion(v)),
    };
    Ok(Migrated {
        settings,
        from_version,
        notes,
    })
}

/// v1 -> v2: split `server_url`, turn `nightly` into a schedule string,
/// and give each folder path its own table.
///
/// # Errors
///
/// `CannotUpgrade` when `server_url` has a port that isn't a number.
pub fn v1_to_v2(old: SettingsV1, notes: &mut Vec<String>) -> Result<SettingsV2, MigrateError> {
    let (host, port) = match old.server_url.rsplit_once(':') {
        Some((host, port)) => {
            let port = port.parse().map_err(|_| {
                MigrateError::CannotUpgrade(format!(
                    "server_url {:?} has a port that isn't a number",
                    old.server_url
                ))
            })?;
            (host.to_string(), port)
        }
        None => {
            notes.push(format!(
                "server_url {:?} had no port: using 443, as version 1 did",
                old.server_url
            ));
            (old.server_url.clone(), 443)
        }
    };
    notes.push(format!(
        "v1 -> v2: server_url {:?} became host {host:?} and port {port}",
        old.server_url
    ));
    let schedule = old.nightly.then(|| {
        notes.push("v1 -> v2: nightly = true became schedule \"daily@02:00\"".to_string());
        "daily@02:00".to_string()
    });
    Ok(SettingsV2 {
        version: 2,
        name: old.name,
        server: crate::legacy::ServerV2 {
            host,
            port,
            timeout_secs: old.timeout_secs,
        },
        schedule,
        folders: old
            .folders
            .into_iter()
            .map(|path| crate::legacy::FolderV2 {
                path,
                include: Vec::new(),
            })
            .collect(),
        keep_versions: 3,
    })
}

/// v2 -> v3: timeouts become durations, the schedule string becomes a
/// table, and port 0 (which v2 never checked) is refused.
///
/// # Errors
///
/// `CannotUpgrade` for port 0 or a schedule string that doesn't parse.
pub fn v2_to_v3(old: SettingsV2, notes: &mut Vec<String>) -> Result<Settings, MigrateError> {
    let port = Port::try_from(old.server.port).map_err(MigrateError::CannotUpgrade)?;
    let timeout = Duration::from_secs(old.server.timeout_secs);
    notes.push(format!(
        "v2 -> v3: timeout_secs = {} became timeout = {:?}",
        old.server.timeout_secs,
        crate::duration::format_duration(timeout)
    ));
    let schedule = match &old.schedule {
        None => Schedule::Manually,
        Some(text) => {
            let schedule = parse_v2_schedule(text).map_err(MigrateError::CannotUpgrade)?;
            notes.push(format!(
                "v2 -> v3: schedule {text:?} became a [schedule] table"
            ));
            schedule
        }
    };
    Ok(Settings {
        version: CURRENT_VERSION,
        name: old.name,
        keep_versions: old.keep_versions,
        server: Server {
            host: old.server.host,
            port,
            timeout,
        },
        schedule,
        folders: old
            .folders
            .into_iter()
            .map(|folder| Folder {
                include: folder.include,
                ..Folder::new(folder.path)
            })
            .collect(),
    })
}

/// `"hourly:15"`, `"daily@02:30"`, `"weekly:fri@18:30"`.
fn parse_v2_schedule(text: &str) -> Result<Schedule, String> {
    let bad = || format!("schedule {text:?} is not hourly:MM, daily@HH:MM or weekly:DAY@HH:MM");
    if let Some(minute) = text.strip_prefix("hourly:") {
        let minute = minute.parse().map_err(|_| bad())?;
        return Ok(Schedule::Hourly { minute });
    }
    if let Some(at) = text.strip_prefix("daily@") {
        return Ok(Schedule::Daily { at: at.parse()? });
    }
    if let Some(rest) = text.strip_prefix("weekly:") {
        let (day, at) = rest.split_once('@').ok_or_else(bad)?;
        // Reuse the Weekday's own Deserialize (with its aliases) by
        // deserializing from a plain string. Any Deserializer will do.
        let on = Weekday::deserialize(
            serde::de::value::StrDeserializer::<serde::de::value::Error>::new(day),
        )
        .map_err(|err| format!("{err} in schedule {text:?}"))?;
        return Ok(Schedule::Weekly {
            on,
            at: at.parse()?,
        });
    }
    Err(bad())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::TimeOfDay;

    const V1: &str = r#"{
        "name": "Family photos",
        "server_url": "backup.example.com:8443",
        "timeout_secs": 20,
        "nightly": true,
        "folders": ["C:/Users/ama/Pictures", "D:/Camera"]
    }"#;

    const V2: &str = r#"{
        "version": 2,
        "name": "Work laptop",
        "server": { "host": "nas.local", "port": 9000, "timeout_secs": 45 },
        "schedule": "weekly:fri@18:30",
        "folders": [ { "path": "C:/Work", "include": ["*.docx"] } ]
    }"#;

    #[test]
    fn upgrades_v1_all_the_way() {
        let migrated = read_any(V1, Format::Json).unwrap();
        assert_eq!(migrated.from_version, 1);
        let s = &migrated.settings;
        assert_eq!(s.version, CURRENT_VERSION);
        assert_eq!(
            (s.server.host.as_str(), s.server.port.get()),
            ("backup.example.com", 8443)
        );
        assert_eq!(s.server.timeout, Duration::from_secs(20));
        assert_eq!(
            s.schedule,
            Schedule::Daily {
                at: TimeOfDay::new(2, 0).unwrap()
            }
        );
        assert_eq!(s.folders[1], Folder::new("D:/Camera"));
        assert_eq!(migrated.notes.len(), 4, "{:#?}", migrated.notes);
    }

    #[test]
    fn upgrades_v2() {
        let migrated = read_any(V2, Format::Json).unwrap();
        assert_eq!(migrated.from_version, 2);
        assert_eq!(
            migrated.settings.schedule,
            Schedule::Weekly {
                on: Weekday::Friday,
                at: TimeOfDay::new(18, 30).unwrap()
            }
        );
        assert_eq!(migrated.settings.folders[0].include, ["*.docx"]);
        assert_eq!(migrated.settings.keep_versions, 3, "v2's default");
    }

    #[test]
    fn current_version_is_read_as_is() {
        let v3 = "version = 3\nname = \"x\"\n[server]\nhost = \"h\"\nport = 1\n";
        let migrated = read_any(v3, Format::Toml).unwrap();
        assert_eq!(migrated.from_version, 3);
        assert!(migrated.notes.is_empty());
    }

    #[test]
    fn refuses_versions_it_does_not_know() {
        assert_eq!(
            read_any("version = 9", Format::Toml),
            Err(MigrateError::TooNew(9))
        );
        assert_eq!(
            read_any(r#"{"version": 0}"#, Format::Json),
            Err(MigrateError::UnknownVersion(0))
        );
    }

    #[test]
    fn explains_values_that_cannot_be_upgraded() {
        let err = read_any(&V1.replace(":8443", ":http"), Format::Json).unwrap_err();
        assert!(err.to_string().contains("isn't a number"), "{err}");
        let err = read_any(&V2.replace("9000", "0"), Format::Json).unwrap_err();
        assert!(err.to_string().contains("port 0"), "{err}");
        let err = read_any(&V2.replace("fri", "someday"), Format::Json).unwrap_err();
        assert!(
            err.to_string().contains("unknown variant `someday`"),
            "{err}"
        );
    }

    #[test]
    fn v1_without_a_port_gets_443_and_a_note() {
        let mut notes = Vec::new();
        let v1: SettingsV1 = serde_json::from_str(&V1.replace(":8443", "")).unwrap();
        let v2 = v1_to_v2(v1, &mut notes).unwrap();
        assert_eq!(v2.server.port, 443);
        assert!(notes[0].contains("had no port"));
    }

    #[test]
    fn v2_schedules() {
        assert_eq!(
            parse_v2_schedule("hourly:15"),
            Ok(Schedule::Hourly { minute: 15 })
        );
        assert!(parse_v2_schedule("weekly:mon@25:00").is_err());
        assert!(parse_v2_schedule("sometimes").is_err());
    }
}
