//! Counting: turning a stream of lines into a [`Summary`].

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::level::Level;
use crate::record::{normalize, parse_line};

/// How many lines had each level. A struct rather than a map, so the JSON
/// always has all six keys in the same order.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelCounts {
    pub trace: u64,
    pub debug: u64,
    pub info: u64,
    pub warn: u64,
    pub error: u64,
    pub fatal: u64,
}

impl LevelCounts {
    /// The count for one level.
    pub fn get(&self, level: Level) -> u64 {
        match level {
            Level::Trace => self.trace,
            Level::Debug => self.debug,
            Level::Info => self.info,
            Level::Warn => self.warn,
            Level::Error => self.error,
            Level::Fatal => self.fatal,
        }
    }

    fn slot(&mut self, level: Level) -> &mut u64 {
        match level {
            Level::Trace => &mut self.trace,
            Level::Debug => &mut self.debug,
            Level::Info => &mut self.info,
            Level::Warn => &mut self.warn,
            Level::Error => &mut self.error,
            Level::Fatal => &mut self.fatal,
        }
    }

    /// The number of lines that had any level.
    pub fn total(&self) -> u64 {
        Level::ALL.iter().map(|&level| self.get(level)).sum()
    }

    /// The number of lines at `level` or more severe.
    pub fn at_or_above(&self, level: Level) -> u64 {
        Level::ALL
            .iter()
            .filter(|&&l| l >= level)
            .map(|&l| self.get(l))
            .sum()
    }
}

/// One of the most frequent messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopMessage {
    pub level: Level,
    pub count: u64,
    /// The message with its numbers replaced by `#`.
    pub pattern: String,
    /// The first real message that had this pattern.
    pub example: String,
}

/// Everything logsum found out. This is what `--json` prints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Summary {
    /// The inputs read, in order (`"-"` is standard input).
    pub inputs: Vec<String>,
    pub lines: u64,
    pub bytes: u64,
    /// Lines with no level (stack traces, blank lines...).
    pub unparsed: u64,
    pub levels: LevelCounts,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    /// The lowest level counted in `top`.
    pub min_level: Level,
    /// The most frequent patterns at `min_level` or above, most frequent first.
    pub top: Vec<TopMessage>,
}

/// Collects the counts while the lines stream past. Only the numbers and
/// one example per pattern are kept, never the whole input, so a 10 GB log
/// needs no more memory than a 10 KB one with the same patterns.
#[derive(Debug)]
pub struct Summarizer {
    inputs: Vec<String>,
    lines: u64,
    bytes: u64,
    unparsed: u64,
    levels: LevelCounts,
    first_timestamp: Option<String>,
    last_timestamp: Option<String>,
    min_level: Level,
    /// (level, pattern) -> (count, first example)
    groups: HashMap<(Level, String), (u64, String)>,
}

impl Summarizer {
    /// A summariser that lists messages at `min_level` or above.
    pub fn new(min_level: Level) -> Self {
        Summarizer {
            inputs: Vec::new(),
            lines: 0,
            bytes: 0,
            unparsed: 0,
            levels: LevelCounts::default(),
            first_timestamp: None,
            last_timestamp: None,
            min_level,
            groups: HashMap::new(),
        }
    }

    /// Records the name of an input that is about to be read.
    pub fn start_input(&mut self, name: impl Into<String>) {
        self.inputs.push(name.into());
    }

    /// Adds the number of bytes read (shown in the summary).
    pub fn add_bytes(&mut self, bytes: u64) {
        self.bytes += bytes;
    }

    /// Counts one line (without its line ending).
    pub fn add_line(&mut self, line: &str) {
        self.lines += 1;
        let Some(record) = parse_line(line) else {
            self.unparsed += 1;
            return;
        };
        *self.levels.slot(record.level) += 1;

        if let Some(ts) = record.timestamp {
            if self.first_timestamp.is_none() {
                self.first_timestamp = Some(ts.to_string());
            }
            // Reuse the String's buffer instead of allocating a new one.
            let last = self.last_timestamp.get_or_insert_with(String::new);
            last.clear();
            last.push_str(ts);
        }

        if record.level >= self.min_level {
            let key = (record.level, normalize(record.message));
            let entry = self
                .groups
                .entry(key)
                .or_insert_with(|| (0, record.message.to_string()));
            entry.0 += 1;
        }
    }

    /// Finishes counting and keeps the `top` most frequent patterns.
    pub fn finish(self, top: usize) -> Summary {
        let mut messages: Vec<TopMessage> = self
            .groups
            .into_iter()
            .map(|((level, pattern), (count, example))| TopMessage {
                level,
                count,
                pattern,
                example,
            })
            .collect();
        // Most frequent first; ties: most severe first, then alphabetical, so
        // the output is the same every run (HashMap order is random).
        messages.sort_by(|a, b| {
            b.count
                .cmp(&a.count)
                .then(b.level.cmp(&a.level))
                .then_with(|| a.pattern.cmp(&b.pattern))
        });
        messages.truncate(top);

        Summary {
            inputs: self.inputs,
            lines: self.lines,
            bytes: self.bytes,
            unparsed: self.unparsed,
            levels: self.levels,
            first_timestamp: self.first_timestamp,
            last_timestamp: self.last_timestamp,
            min_level: self.min_level,
            top: messages,
        }
    }
}

/// Summarises some text in one go: handy in tests and examples.
///
/// ```
/// use capstone_cli_tool::{Level, summarize_text};
/// let summary = summarize_text("INFO up\nERROR down 1\nERROR down 2\n", Level::Warn, 5);
/// assert_eq!(summary.levels.error, 2);
/// assert_eq!(summary.top[0].pattern, "down #");
/// ```
pub fn summarize_text(text: &str, min_level: Level, top: usize) -> Summary {
    let mut summarizer = Summarizer::new(min_level);
    summarizer.start_input("<text>");
    summarizer.add_bytes(text.len() as u64);
    for line in text.lines() {
        summarizer.add_line(line);
    }
    summarizer.finish(top)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOG: &str = "\
2026-05-01T09:00:00Z INFO start
2026-05-01T09:00:01Z WARN slow query took 1532ms
2026-05-01T09:00:02Z ERROR payment 991 failed
    at billing::charge
2026-05-01T09:00:03Z ERROR payment 17 failed
2026-05-01T09:00:04Z WARN slow query took 87ms
2026-05-01T09:00:05Z WARN slow query took 3ms
2026-05-01T09:00:06Z DEBUG cache hit
";

    #[test]
    fn counts_lines_levels_and_unparsed() {
        let s = summarize_text(LOG, Level::Warn, 5);
        assert_eq!(s.lines, 8);
        assert_eq!(s.unparsed, 1);
        assert_eq!(s.levels.info, 1);
        assert_eq!(s.levels.warn, 3);
        assert_eq!(s.levels.error, 2);
        assert_eq!(s.levels.debug, 1);
        assert_eq!(s.levels.total(), 7);
        assert_eq!(s.levels.at_or_above(Level::Warn), 5);
    }

    #[test]
    fn remembers_first_and_last_timestamp() {
        let s = summarize_text(LOG, Level::Warn, 5);
        assert_eq!(s.first_timestamp.as_deref(), Some("2026-05-01T09:00:00Z"));
        assert_eq!(s.last_timestamp.as_deref(), Some("2026-05-01T09:00:06Z"));
    }

    #[test]
    fn groups_by_pattern_most_frequent_first() {
        let s = summarize_text(LOG, Level::Warn, 5);
        let top: Vec<_> = s
            .top
            .iter()
            .map(|m| (m.count, m.level, m.pattern.as_str(), m.example.as_str()))
            .collect();
        assert_eq!(
            top,
            vec![
                (
                    3,
                    Level::Warn,
                    "slow query took #ms",
                    "slow query took 1532ms"
                ),
                (2, Level::Error, "payment # failed", "payment 991 failed"),
            ]
        );
    }

    #[test]
    fn min_level_and_top_limit_what_is_listed() {
        let s = summarize_text(LOG, Level::Error, 5);
        assert_eq!(s.top.len(), 1);
        assert_eq!(s.top[0].level, Level::Error);

        let s = summarize_text(LOG, Level::Trace, 1);
        assert_eq!(s.top.len(), 1);
        assert_eq!(s.top[0].count, 3);
    }

    #[test]
    fn ties_are_ordered_the_same_every_time() {
        let s = summarize_text("ERROR b\nWARN a\nERROR a\n", Level::Warn, 5);
        let patterns: Vec<_> = s
            .top
            .iter()
            .map(|m| (m.level, m.pattern.as_str()))
            .collect();
        assert_eq!(
            patterns,
            vec![(Level::Error, "a"), (Level::Error, "b"), (Level::Warn, "a")]
        );
    }
}
