//! The old file formats, frozen exactly as they were released.
//!
//! Rule of thumb: once a format has shipped, its structs never change
//! again. New ideas go in a new version, plus a migration step
//! ([`crate::migrate`]). Old formats are only ever *read*, so these types
//! derive `Deserialize` but not `Serialize`.

use serde::Deserialize;

/// Version 1 (2019): JSON only, and no `version` field at all.
///
/// ```json
/// {
///   "name": "Family photos",
///   "server_url": "backup.example.com:8443",
///   "timeout_secs": 20,
///   "nightly": true,
///   "folders": ["C:/Users/ama/Pictures", "D:/Camera"]
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SettingsV1 {
    /// The backup set's name.
    pub name: String,
    /// `host:port`, or just `host` (then the port was 443).
    pub server_url: String,
    /// The timeout in whole seconds.
    #[serde(default = "v1_timeout")]
    pub timeout_secs: u64,
    /// `true` meant "every night at 02:00".
    #[serde(default)]
    pub nightly: bool,
    /// Plain paths: v1 had no patterns.
    #[serde(default)]
    pub folders: Vec<String>,
}

fn v1_timeout() -> u64 {
    60
}

/// Version 2 (2022): the server became a table, and schedules a string.
///
/// ```json
/// {
///   "version": 2,
///   "name": "Work laptop",
///   "server": { "host": "nas.local", "port": 9000, "timeout_secs": 45 },
///   "schedule": "weekly:fri@18:30",
///   "folders": [ { "path": "C:/Work", "include": ["*.docx"] } ]
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SettingsV2 {
    /// Always 2.
    pub version: u32,
    /// The backup set's name.
    pub name: String,
    /// The server, now with separate fields.
    pub server: ServerV2,
    /// `"hourly:15"`, `"daily@02:30"`, `"weekly:fri@18:30"`, or missing.
    #[serde(default)]
    pub schedule: Option<String>,
    /// Folders, now with patterns.
    #[serde(default)]
    pub folders: Vec<FolderV2>,
    /// v2 used snake_case.
    #[serde(default = "v2_keep")]
    pub keep_versions: u8,
}

fn v2_keep() -> u8 {
    3
}

/// The server table in version 2.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ServerV2 {
    /// Host name.
    pub host: String,
    /// Port number (0 was not checked in v2!).
    pub port: u16,
    /// Timeout in whole seconds.
    #[serde(default = "v1_timeout")]
    pub timeout_secs: u64,
}

/// A folder in version 2.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct FolderV2 {
    /// The folder's path.
    pub path: String,
    /// File patterns.
    #[serde(default)]
    pub include: Vec<String>,
}
