//! Understanding one log line: its timestamp, level and message.

use crate::level::Level;

/// One parsed log line. It borrows from the line, so parsing allocates
/// nothing (lifetimes, lesson 03/01).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Record<'a> {
    /// The text before the level, if it looks like a date or time.
    pub timestamp: Option<&'a str>,
    /// How serious the line is.
    pub level: Level,
    /// Everything after the level.
    pub message: &'a str,
}

/// How many words at the start of a line may come before the level. Looking
/// further would mistake "the user typed error" for an error line.
const MAX_WORDS_BEFORE_LEVEL: usize = 3;

/// Parses the common "timestamp LEVEL message" shapes:
///
/// ```text
/// 2026-05-01T09:00:00Z INFO server started
/// 2026-05-01 09:00:03 WARN [db] slow query took 1532ms
/// [2026-05-01 09:00:05] [error] payment 991 failed
/// ERROR: disk full
/// ```
///
/// Returns `None` for lines without a level, such as stack-trace lines.
///
/// ```
/// use capstone_cli_tool::{Level, parse_line};
/// let record = parse_line("2026-05-01 09:00:03 WARN slow query").unwrap();
/// assert_eq!(record.timestamp, Some("2026-05-01 09:00:03"));
/// assert_eq!(record.level, Level::Warn);
/// assert_eq!(record.message, "slow query");
/// ```
pub fn parse_line(line: &str) -> Option<Record<'_>> {
    for (start, word) in words(line).take(MAX_WORDS_BEFORE_LEVEL + 1) {
        let bare = word.trim_matches(|c| matches!(c, '[' | ']' | '(' | ')' | '<' | '>' | ':'));
        let Ok(level) = bare.parse::<Level>() else {
            continue;
        };
        let before = line[..start].trim().trim_matches(['[', ']']).trim();
        let timestamp = before
            .starts_with(|c: char| c.is_ascii_digit())
            .then_some(before);
        let message = line[start + word.len()..]
            .trim_start()
            .trim_start_matches(['-', ':'])
            .trim();
        return Some(Record {
            timestamp,
            level,
            message,
        });
    }
    None
}

/// The whitespace-separated words of `line`, each with its byte offset.
fn words(line: &str) -> impl Iterator<Item = (usize, &str)> + '_ {
    let mut offset = 0;
    let mut rest = line;
    std::iter::from_fn(move || {
        let trimmed = rest.trim_start();
        offset += rest.len() - trimmed.len();
        if trimmed.is_empty() {
            return None;
        }
        let end = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
        let found = (offset, &trimmed[..end]);
        offset += end;
        rest = &trimmed[end..];
        Some(found)
    })
}

/// Turns a message into a *pattern*, so that messages that differ only in
/// their numbers are counted together: every run of digits becomes `#`.
///
/// ```
/// use capstone_cli_tool::normalize;
/// assert_eq!(normalize("slow query took 1532ms"), "slow query took #ms");
/// assert_eq!(normalize("payment 991 failed"), normalize("payment 17 failed"));
/// ```
pub fn normalize(message: &str) -> String {
    let mut pattern = String::with_capacity(message.len());
    let mut in_number = false;
    for c in message.chars() {
        if c.is_ascii_digit() {
            if !in_number {
                pattern.push('#');
            }
            in_number = true;
        } else {
            pattern.push(c);
            in_number = false;
        }
    }
    pattern
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(line: &str) -> (Option<&str>, Level, &str) {
        let r = parse_line(line).expect("a log line");
        (r.timestamp, r.level, r.message)
    }

    #[test]
    fn iso_timestamp_then_level() {
        assert_eq!(
            parsed("2026-05-01T09:00:00Z INFO server started on port 8080"),
            (
                Some("2026-05-01T09:00:00Z"),
                Level::Info,
                "server started on port 8080"
            )
        );
    }

    #[test]
    fn date_and_time_as_two_words() {
        assert_eq!(
            parsed("2026-05-01 09:00:03 WARN [db] slow query"),
            (Some("2026-05-01 09:00:03"), Level::Warn, "[db] slow query")
        );
    }

    #[test]
    fn brackets_and_lowercase_level() {
        assert_eq!(
            parsed("[2026-05-01 09:00:05] [error] payment 991 failed"),
            (
                Some("2026-05-01 09:00:05"),
                Level::Error,
                "payment 991 failed"
            )
        );
    }

    #[test]
    fn level_first_with_colon_and_no_timestamp() {
        assert_eq!(
            parsed("ERROR: disk full"),
            (None, Level::Error, "disk full")
        );
        assert_eq!(
            parsed("myapp warning - cache cold"),
            (None, Level::Warn, "cache cold")
        );
    }

    #[test]
    fn lines_without_a_level_are_not_records() {
        assert_eq!(parse_line(""), None);
        assert_eq!(parse_line("    at main (src/main.rs:12)"), None);
        // "error" is the 5th word: too far in to be the line's level.
        assert_eq!(parse_line("the user typed the error twice"), None);
    }

    #[test]
    fn words_reports_byte_offsets() {
        let found: Vec<_> = words("  ab  c").collect();
        assert_eq!(found, vec![(2, "ab"), (6, "c")]);
    }

    #[test]
    fn normalize_replaces_each_run_of_digits() {
        assert_eq!(
            normalize("retry 3 of 10 after 250ms"),
            "retry # of # after #ms"
        );
        assert_eq!(normalize("no numbers"), "no numbers");
        assert_eq!(normalize("ünïcode 42 ok"), "ünïcode # ok");
    }
}
