//! Reading input of any size, one line at a time.

use std::borrow::Cow;
use std::io::{self, BufRead};

/// Calls `on_line(line, bytes_so_far)` for every line of `reader`, and
/// returns the total number of bytes read.
///
/// Why not `reader.lines()`?
/// - `lines()` allocates a new `String` for every line. This reuses ONE
///   buffer for the whole input, so a 10 GB file costs no more memory than
///   its longest line.
/// - `lines()` stops with an error at the first byte that isn't valid UTF-8,
///   and real logs often contain some. This replaces bad bytes with `�`
///   (`String::from_utf8_lossy`) and keeps going.
///
/// Line endings (`\n` or `\r\n`) are removed before `on_line` sees the line.
///
/// ```
/// use capstone_cli_tool::for_each_line;
/// let mut seen = Vec::new();
/// let bytes = for_each_line(&b"one\r\ntwo\nbad \xFF byte"[..], |line, _| seen.push(line.to_string()))?;
/// assert_eq!(seen, ["one", "two", "bad \u{FFFD} byte"]);
/// assert_eq!(bytes, 19);
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// # Errors
///
/// Any I/O error from `reader` (other than `Interrupted`, which is retried).
pub fn for_each_line<R: BufRead>(
    mut reader: R,
    mut on_line: impl FnMut(&str, u64),
) -> io::Result<u64> {
    let mut buf = Vec::with_capacity(256);
    let mut total = 0u64;
    loop {
        buf.clear();
        let n = match reader.read_until(b'\n', &mut buf) {
            Ok(0) => return Ok(total), // end of input
            Ok(n) => n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        total += n as u64;
        let line = trim_line_ending(&buf);
        let text: Cow<'_, str> = String::from_utf8_lossy(line);
        on_line(&text, total);
    }
}

/// Removes a trailing `\n` or `\r\n`.
fn trim_line_ending(line: &[u8]) -> &[u8] {
    let line = line.strip_suffix(b"\n").unwrap_or(line);
    line.strip_suffix(b"\r").unwrap_or(line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Read};

    fn collect(bytes: &[u8]) -> (Vec<String>, u64) {
        let mut lines = Vec::new();
        let total = for_each_line(bytes, |line, _| lines.push(line.to_string())).unwrap();
        (lines, total)
    }

    #[test]
    fn handles_both_line_endings_and_a_missing_last_newline() {
        let (lines, total) = collect(b"a\r\nb\nc");
        assert_eq!(lines, ["a", "b", "c"]);
        assert_eq!(total, 6);
    }

    #[test]
    fn keeps_empty_lines_and_empty_input() {
        assert_eq!(collect(b"\n\nx\n").0, ["", "", "x"]);
        assert_eq!(collect(b""), (Vec::<String>::new(), 0));
    }

    #[test]
    fn invalid_utf8_is_replaced_not_fatal() {
        let (lines, _) = collect(b"ok\n\xFF\xFEbroken\nfine\n");
        assert_eq!(lines, ["ok", "\u{FFFD}\u{FFFD}broken", "fine"]);

        // For comparison: `lines()` gives up at the bad line.
        let mut results = (&b"ok\n\xFF\xFEbroken\nfine\n"[..]).lines();
        assert!(results.next().unwrap().is_ok());
        assert_eq!(
            results.next().unwrap().unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn reports_running_byte_totals() {
        let mut totals = Vec::new();
        for_each_line(&b"ab\ncd\n"[..], |_, so_far| totals.push(so_far)).unwrap();
        assert_eq!(totals, [3, 6]);
    }

    #[test]
    fn works_through_a_tiny_buffer() {
        // A 2-byte BufReader forces lines to arrive in pieces.
        let reader = BufReader::with_capacity(2, &b"hello\nworld\n"[..]);
        let mut lines = Vec::new();
        for_each_line(reader, |line, _| lines.push(line.to_string())).unwrap();
        assert_eq!(lines, ["hello", "world"]);
    }

    /// A reader that fails half way, like a disk error or a dropped network share.
    struct Failing;
    impl Read for Failing {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("disk on fire"))
        }
    }

    #[test]
    fn passes_real_errors_on() {
        let err = for_each_line(BufReader::new(Failing), |_, _| {}).unwrap_err();
        assert_eq!(err.to_string(), "disk on fire");
    }
}
