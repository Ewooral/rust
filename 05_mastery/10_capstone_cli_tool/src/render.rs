//! Output for people (an aligned table) and for programs (JSON).

use std::io::{self, Write};

use crate::level::Level;
use crate::summary::Summary;
use crate::term::{Palette, human_bytes};

/// The longest message shown in the table, in characters.
const MAX_MESSAGE_CHARS: usize = 100;

/// Writes the summary as an aligned table for people to read.
///
/// # Errors
///
/// Any error from `out`, such as a closed pipe.
pub fn write_table(out: &mut impl Write, summary: &Summary, palette: Palette) -> io::Result<()> {
    let inputs = match summary.inputs.len() {
        1 => "1 input".to_string(),
        n => format!("{n} inputs"),
    };
    writeln!(
        out,
        "{} {inputs}, {} lines ({}), {} without a level",
        palette.bold("logsum:"),
        group_digits(summary.lines),
        human_bytes(summary.bytes),
        group_digits(summary.unparsed),
    )?;
    if let (Some(first), Some(last)) = (&summary.first_timestamp, &summary.last_timestamp) {
        writeln!(out, "{} {first} .. {last}", palette.bold("time:  "))?;
    }

    // Levels, most severe first. Columns are as wide as their widest value.
    let counted = summary.levels.total();
    let width = Level::ALL
        .iter()
        .map(|&l| group_digits(summary.levels.get(l)).len())
        .max()
        .unwrap_or(0)
        .max("COUNT".len());
    writeln!(out)?;
    writeln!(
        out,
        "{}",
        palette.bold(&format!(
            "{:<5}  {:>width$}  {:>6}",
            "LEVEL", "COUNT", "SHARE"
        ))
    )?;
    for &level in Level::ALL.iter().rev() {
        let count = summary.levels.get(level);
        let share = if counted == 0 {
            0.0
        } else {
            count as f64 * 100.0 / counted as f64
        };
        // Pad first, then paint: escape codes would upset the padding.
        let name = palette.level(level, &format!("{level:<5}"));
        writeln!(
            out,
            "{name}  {:>width$}  {:>5.1}%",
            group_digits(count),
            share
        )?;
    }

    writeln!(out)?;
    writeln!(
        out,
        "{}",
        palette.bold(&format!(
            "TOP MESSAGES ({} and above)",
            summary.min_level.as_str()
        ))
    )?;
    if summary.top.is_empty() {
        writeln!(out, "{}", palette.dim("(none)"))?;
        return Ok(());
    }
    let width = summary
        .top
        .iter()
        .map(|m| group_digits(m.count).len())
        .max()
        .unwrap_or(0)
        .max("COUNT".len());
    writeln!(
        out,
        "{}",
        palette.bold(&format!("{:>width$}  {:<5}  MESSAGE", "COUNT", "LEVEL"))
    )?;
    for message in &summary.top {
        let level = palette.level(message.level, &format!("{:<5}", message.level));
        writeln!(
            out,
            "{:>width$}  {level}  {}",
            group_digits(message.count),
            truncate(&message.pattern, MAX_MESSAGE_CHARS)
        )?;
    }
    Ok(())
}

/// Writes the summary as pretty-printed JSON, for programs (`jq`, scripts,
/// dashboards). The shape is the [`Summary`] struct, field for field.
///
/// # Errors
///
/// Any error from `out`.
pub fn write_json(out: &mut impl Write, summary: &Summary) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut *out, summary)?; // serde_json::Error converts into io::Error
    writeln!(out)
}

/// `1234567` -> `"1,234,567"`.
///
/// ```
/// use capstone_cli_tool::group_digits;
/// assert_eq!(group_digits(999), "999");
/// assert_eq!(group_digits(1234567), "1,234,567");
/// ```
pub fn group_digits(n: u64) -> String {
    let digits = n.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(c);
    }
    grouped
}

/// Shortens `text` to at most `max` characters, ending with `…` if cut.
/// Counts characters, not bytes, so it never cuts a letter in half.
fn truncate(text: &str, max: usize) -> String {
    match text.char_indices().nth(max.saturating_sub(1)) {
        Some((cut, _)) if text[cut..].chars().count() > 1 => format!("{}…", &text[..cut]),
        _ => text.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::summary::summarize_text;

    const LOG: &str = "\
2026-05-01T09:00:00Z INFO start
2026-05-01T09:00:01Z WARN slow query took 1532ms
2026-05-01T09:00:02Z ERROR payment 991 failed
2026-05-01T09:00:04Z WARN slow query took 87ms
not a log line
";

    fn table(colour: bool) -> String {
        let summary = summarize_text(LOG, Level::Warn, 5);
        let mut out = Vec::new();
        write_table(&mut out, &summary, Palette::new(colour)).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn plain_table_is_aligned() {
        let expected = "\
logsum: 1 input, 5 lines (189 B), 1 without a level
time:   2026-05-01T09:00:00Z .. 2026-05-01T09:00:04Z

LEVEL  COUNT   SHARE
fatal      0    0.0%
error      1   25.0%
warn       2   50.0%
info       1   25.0%
debug      0    0.0%
trace      0    0.0%

TOP MESSAGES (warn and above)
COUNT  LEVEL  MESSAGE
    2  warn   slow query took #ms
    1  error  payment # failed
";
        assert_eq!(table(false), expected);
    }

    #[test]
    fn coloured_table_has_escape_codes_but_the_same_text() {
        let coloured = table(true);
        assert!(coloured.contains("\x1b[31merror\x1b[0m"));
        let stripped = strip_ansi(&coloured);
        assert_eq!(stripped, table(false));
    }

    #[test]
    fn json_round_trips() {
        let summary = summarize_text(LOG, Level::Warn, 5);
        let mut out = Vec::new();
        write_json(&mut out, &summary).unwrap();
        let back: Summary = serde_json::from_slice(&out).unwrap();
        assert_eq!(back, summary);
        let value: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(value["levels"]["warn"], 2);
        assert_eq!(value["top"][0]["pattern"], "slow query took #ms");
    }

    #[test]
    fn empty_input_says_none() {
        let summary = summarize_text("", Level::Warn, 5);
        let mut out = Vec::new();
        write_table(&mut out, &summary, Palette::new(false)).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(
            text.ends_with("TOP MESSAGES (warn and above)\n(none)\n"),
            "{text}"
        );
        assert!(!text.contains("time:"));
    }

    #[test]
    fn group_digits_and_truncate() {
        assert_eq!(group_digits(0), "0");
        assert_eq!(group_digits(1000), "1,000");
        assert_eq!(truncate("abcdef", 4), "abc…");
        assert_eq!(truncate("abcd", 4), "abcd");
        assert_eq!(truncate("héllo wörld", 5), "héll…");
    }

    fn strip_ansi(text: &str) -> String {
        let mut out = String::new();
        let mut chars = text.chars();
        while let Some(c) = chars.next() {
            if c == '\x1b' {
                for c in chars.by_ref() {
                    if c == 'm' {
                        break;
                    }
                }
            } else {
                out.push(c);
            }
        }
        out
    }
}
