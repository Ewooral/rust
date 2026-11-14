//! Exercise 4: extend logsum to read JSON logs ("JSON lines": one JSON
//! object per line), which many services write instead of plain text:
//!
//!   {"ts":"2026-05-01T09:00:00Z","level":"error","msg":"payment 991 failed"}
//!
//! 1. `JsonLine` is the shape of one line. Different programs use different
//!    key names: make serde also accept `timestamp` for `ts` and `message`
//!    for `msg` (look up `#[serde(alias = "...")]`). `ts` may be missing.
//! 2. `parse_json_line`: `None` if the line isn't a JSON object of that
//!    shape, or its level isn't a level logsum knows (use `Level`'s
//!    `FromStr`, so `"WARNING"` works too).
//! 3. `summarize_json_lines`: summarise JSON-lines text by turning each
//!    parsed line back into `"<ts> <LEVEL> <msg>"` (or `"<LEVEL> <msg>"`)
//!    and feeding the library's `Summarizer`. Lines that don't parse are
//!    still counted (pass them through unchanged: they become "unparsed").
//!    This "adapter" reuses all of logsum's counting without changing it.
//!
//! Test: cargo test -p capstone_cli_tool --features exercises --bin ex04_json_lines_input

#![allow(unused_variables, unused_imports, dead_code)]

use capstone_cli_tool::{Level, Summarizer, Summary};
use serde::Deserialize;

/// One line of a JSON log.
#[derive(Debug, Deserialize)]
struct JsonLine {
    ts: Option<String>,
    level: String,
    msg: String,
}

/// A parsed JSON log line: (timestamp, level, message).
fn parse_json_line(line: &str) -> Option<(Option<String>, Level, String)> {
    todo!()
}

/// Summarises JSON-lines text with logsum's own Summarizer.
fn summarize_json_lines(text: &str, min_level: Level, top: usize) -> Summary {
    todo!()
}

fn main() {
    let log = r#"{"ts":"2026-05-01T09:00:00Z","level":"info","msg":"server started"}
{"ts":"2026-05-01T09:00:02Z","level":"error","msg":"payment 991 failed"}
{"timestamp":"2026-05-01T09:00:03Z","level":"ERROR","message":"payment 17 failed"}
not json at all
"#;
    let summary = summarize_json_lines(log, Level::Warn, 5);
    println!("{}", serde_json::to_string_pretty(&summary).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_usual_keys() {
        assert_eq!(
            parse_json_line(
                r#"{"ts":"2026-05-01T09:00:00Z","level":"warn","msg":"disk 91% full"}"#
            ),
            Some((
                Some("2026-05-01T09:00:00Z".to_string()),
                Level::Warn,
                "disk 91% full".to_string()
            ))
        );
    }

    #[test]
    fn accepts_aliases_a_missing_timestamp_and_any_level_spelling() {
        assert_eq!(
            parse_json_line(r#"{"timestamp":"t1","level":"WARNING","message":"m"}"#),
            Some((Some("t1".to_string()), Level::Warn, "m".to_string()))
        );
        assert_eq!(
            parse_json_line(r#"{"level":"fatal","msg":"boom"}"#),
            Some((None, Level::Fatal, "boom".to_string()))
        );
    }

    #[test]
    fn rejects_what_it_cannot_use() {
        assert_eq!(parse_json_line("not json"), None);
        assert_eq!(parse_json_line(r#"{"level":"loud","msg":"x"}"#), None);
        assert_eq!(parse_json_line(r#"{"msg":"no level"}"#), None);
        assert_eq!(parse_json_line(r#"[1,2,3]"#), None);
    }

    #[test]
    fn summarises_with_the_library() {
        let log = concat!(
            r#"{"ts":"2026-05-01T09:00:00Z","level":"info","msg":"server started"}"#,
            "\n",
            r#"{"ts":"2026-05-01T09:00:02Z","level":"error","msg":"payment 991 failed"}"#,
            "\n",
            r#"{"timestamp":"2026-05-01T09:00:03Z","level":"ERROR","message":"payment 17 failed"}"#,
            "\n",
            "not json at all\n",
        );
        let s = summarize_json_lines(log, Level::Warn, 5);
        assert_eq!(s.lines, 4);
        assert_eq!(s.unparsed, 1);
        assert_eq!(s.levels.info, 1);
        assert_eq!(s.levels.error, 2);
        assert_eq!(s.first_timestamp.as_deref(), Some("2026-05-01T09:00:00Z"));
        assert_eq!(s.last_timestamp.as_deref(), Some("2026-05-01T09:00:03Z"));
        assert_eq!(s.top[0].pattern, "payment # failed");
        assert_eq!(s.top[0].count, 2);
    }
}
