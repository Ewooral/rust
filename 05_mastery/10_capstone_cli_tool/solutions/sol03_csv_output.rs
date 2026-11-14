//! Solution to exercise 3: extend logsum with a third output format, CSV,
//! for spreadsheets.
//!
//! Implement:
//! - `csv_field`: a value as one CSV field. Most values are written as they
//!   are. A value containing a comma, a double quote, a `\r` or a `\n` must
//!   be wrapped in double quotes, with every `"` inside doubled (RFC 4180):
//!   `say "hi", then go`  ->  `"say ""hi"", then go"`
//! - `write_csv`: the summary's top messages as CSV, one row each, under a
//!   header row `level,count,pattern,example`. Lines end with `\n`.
//!
//! In a real project you would reach for the `csv` crate, which also reads
//! CSV; writing it by hand once shows what "quoting" means.
//!
//! Test: cargo test -p capstone_cli_tool --features solutions --bin sol03_csv_output

use std::borrow::Cow;
use std::io::{self, Write};

use capstone_cli_tool::{Level, Summary, summarize_text};

/// One CSV field, quoted only if it has to be.
fn csv_field(value: &str) -> Cow<'_, str> {
    if value.contains([',', '"', '\r', '\n']) {
        Cow::Owned(format!("\"{}\"", value.replace('"', "\"\"")))
    } else {
        Cow::Borrowed(value)
    }
}

/// The top messages as CSV.
fn write_csv(out: &mut impl Write, summary: &Summary) -> io::Result<()> {
    writeln!(out, "level,count,pattern,example")?;
    for message in &summary.top {
        writeln!(
            out,
            "{},{},{},{}",
            message.level,
            message.count,
            csv_field(&message.pattern),
            csv_field(&message.example)
        )?;
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let log = "\
ERROR payment 991 failed: card \"declined\"
ERROR payment 17 failed: card \"declined\"
WARN disk 91% full, cleaning up
";
    let summary = summarize_text(log, Level::Warn, 5);
    write_csv(&mut io::stdout().lock(), &summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_values_are_borrowed_unchanged() {
        assert_eq!(csv_field("payment # failed"), "payment # failed");
        assert!(matches!(csv_field("abc"), Cow::Borrowed(_)));
        assert_eq!(csv_field(""), "");
    }

    #[test]
    fn special_characters_are_quoted_and_quotes_doubled() {
        assert_eq!(csv_field("a,b"), "\"a,b\"");
        assert_eq!(csv_field("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(csv_field("two\nlines"), "\"two\nlines\"");
        assert_eq!(
            csv_field("say \"hi\", then go"),
            "\"say \"\"hi\"\", then go\""
        );
    }

    #[test]
    fn writes_a_header_and_one_row_per_top_message() {
        let log = "ERROR payment 991 failed\nERROR payment 17 failed\nWARN disk 91% full, cleaning up\nINFO ok\n";
        let summary = summarize_text(log, Level::Warn, 5);
        let mut out = Vec::new();
        write_csv(&mut out, &summary).unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "level,count,pattern,example\n\
             error,2,payment # failed,payment 991 failed\n\
             warn,1,\"disk #% full, cleaning up\",\"disk 91% full, cleaning up\"\n"
        );
    }
}
