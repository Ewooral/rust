//! Checks that need the *whole* settings, after parsing.
//!
//! Two layers of checking:
//!
//! 1. **While parsing**, the types refuse impossible values: a `Port` can't
//!    be 0 and a `TimeOfDay` can't be 25:00 (`try_from`). Those errors come
//!    with a line and column.
//! 2. **After parsing**, [`validate`] looks at the settings as a whole and
//!    reports *every* problem at once, so the user can fix them in one go.

use std::collections::HashSet;
use std::fmt;
use std::time::Duration;

use crate::settings::{Schedule, Settings};

/// One thing wrong with the settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// Where, as a dotted path: `server.host`, `folders[1].path`.
    pub field: String,
    /// What is wrong, in words a user understands.
    pub message: String,
}

impl Problem {
    fn new(field: impl Into<String>, message: impl Into<String>) -> Problem {
        Problem {
            field: field.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// The shortest and longest timeouts that make sense.
pub const TIMEOUT_RANGE: (Duration, Duration) = (Duration::from_secs(1), Duration::from_secs(600));

/// Every problem with `settings`; empty means they're fine.
pub fn validate(settings: &Settings) -> Vec<Problem> {
    let mut problems = Vec::new();

    if settings.name.trim().is_empty() {
        problems.push(Problem::new("name", "must not be empty"));
    }
    if !(1..=100).contains(&settings.keep_versions) {
        problems.push(Problem::new(
            "keep-versions",
            format!("must be 1 to 100, not {}", settings.keep_versions),
        ));
    }

    let host = &settings.server.host;
    if host.is_empty() {
        problems.push(Problem::new("server.host", "must not be empty"));
    } else if host.contains("://") {
        problems.push(Problem::new(
            "server.host",
            format!("{host:?} is a URL; write only the host name"),
        ));
    } else if host.contains(char::is_whitespace) {
        problems.push(Problem::new("server.host", "must not contain spaces"));
    }

    let (min, max) = TIMEOUT_RANGE;
    if !(min..=max).contains(&settings.server.timeout) {
        problems.push(Problem::new(
            "server.timeout",
            format!(
                "must be between {} and {}",
                crate::duration::format_duration(min),
                crate::duration::format_duration(max)
            ),
        ));
    }

    if let Schedule::Hourly { minute } = settings.schedule
        && minute > 59
    {
        problems.push(Problem::new(
            "schedule.minute",
            format!("must be 0 to 59, not {minute}"),
        ));
    }

    if settings.folders.is_empty() {
        problems.push(Problem::new(
            "folders",
            "add at least one folder to back up",
        ));
    }
    let mut seen = HashSet::new();
    for (index, folder) in settings.folders.iter().enumerate() {
        let field = format!("folders[{index}].path");
        if folder.path.trim().is_empty() {
            problems.push(Problem::new(field, "must not be empty"));
        } else if !seen.insert(folder.path.to_lowercase()) {
            problems.push(Problem::new(
                field,
                format!("{:?} is listed twice", folder.path),
            ));
        }
    }

    problems
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{Folder, Port, Server};

    fn good() -> Settings {
        Settings {
            version: 3,
            name: "Photos".to_string(),
            keep_versions: 5,
            server: Server {
                host: "backup.example.com".to_string(),
                port: Port::try_from(8443).unwrap(),
                timeout: Duration::from_secs(30),
            },
            schedule: Schedule::Manually,
            folders: vec![Folder::new("C:/Pictures")],
        }
    }

    #[test]
    fn good_settings_have_no_problems() {
        assert_eq!(validate(&good()), vec![]);
    }

    #[test]
    fn reports_every_problem_at_once() {
        let mut settings = good();
        settings.name = "  ".to_string();
        settings.server.host = "https://backup.example.com".to_string();
        settings.server.timeout = Duration::ZERO;
        settings.keep_versions = 0;
        settings.schedule = Schedule::Hourly { minute: 75 };
        settings.folders = vec![Folder::new("C:/A"), Folder::new("c:/a"), Folder::new("")];
        let fields: Vec<String> = validate(&settings).into_iter().map(|p| p.field).collect();
        assert_eq!(
            fields,
            [
                "name",
                "keep-versions",
                "server.host",
                "server.timeout",
                "schedule.minute",
                "folders[1].path",
                "folders[2].path"
            ]
        );
    }

    #[test]
    fn no_folders_is_a_problem() {
        let mut settings = good();
        settings.folders.clear();
        let problems = validate(&settings);
        assert_eq!(problems.len(), 1);
        assert_eq!(
            problems[0].to_string(),
            "folders: add at least one folder to back up"
        );
    }
}
