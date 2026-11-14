//! Being a good terminal citizen: colours and a progress line only when a
//! person is watching, never when the output goes to a file or a pipe.

use std::fmt;
use std::io::Write;
use std::str::FromStr;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::level::Level;

/// The `--color` setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorWhen {
    /// Colour only when stdout is a terminal and `NO_COLOR` isn't set.
    Auto,
    /// Always colour, even into a pipe (for `less -R`).
    Always,
    /// Never colour.
    Never,
}

impl fmt::Display for ColorWhen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            ColorWhen::Auto => "auto",
            ColorWhen::Always => "always",
            ColorWhen::Never => "never",
        })
    }
}

/// The error for text that isn't `auto`, `always` or `never`.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unknown colour choice {0:?} (expected auto, always or never)")]
pub struct ParseColorError(pub String);

impl FromStr for ColorWhen {
    type Err = ParseColorError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text.to_ascii_lowercase().as_str() {
            "auto" => Ok(ColorWhen::Auto),
            "always" => Ok(ColorWhen::Always),
            "never" => Ok(ColorWhen::Never),
            _ => Err(ParseColorError(text.to_string())),
        }
    }
}

/// Decides whether to colour the output.
///
/// - `always` / `never` do what they say.
/// - `auto` colours only if stdout `is_terminal`, `NO_COLOR` is unset or
///   empty (<https://no-color.org>), and `TERM` isn't `dumb`.
///
/// The environment is passed in as plain values, so this is a pure function
/// that tests can call with every combination.
///
/// ```
/// use capstone_cli_tool::{ColorWhen, use_color};
/// assert!(use_color(ColorWhen::Auto, true, None, None));
/// assert!(!use_color(ColorWhen::Auto, false, None, None)); // piped
/// assert!(!use_color(ColorWhen::Auto, true, Some("1"), None)); // NO_COLOR=1
/// assert!(use_color(ColorWhen::Always, false, Some("1"), None));
/// ```
pub fn use_color(
    when: ColorWhen,
    is_terminal: bool,
    no_color: Option<&str>,
    term: Option<&str>,
) -> bool {
    match when {
        ColorWhen::Always => true,
        ColorWhen::Never => false,
        ColorWhen::Auto => {
            is_terminal && no_color.is_none_or(str::is_empty) && term != Some("dumb")
        }
    }
}

/// Paints text with ANSI escape codes, or leaves it alone when disabled.
///
/// An escape code is a few invisible bytes such as `ESC[31m` ("red from
/// here") and `ESC[0m` ("back to normal"). Crates such as `anstream` and
/// `owo-colors` do this for real programs; it's done by hand here so you
/// can see there's no magic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    enabled: bool,
}

impl Palette {
    /// A palette that colours if `enabled`.
    pub fn new(enabled: bool) -> Self {
        Palette { enabled }
    }

    /// Whether this palette adds colour.
    pub fn enabled(self) -> bool {
        self.enabled
    }

    fn paint(self, code: &str, text: &str) -> String {
        if self.enabled {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }

    /// Bold, for headings.
    pub fn bold(self, text: &str) -> String {
        self.paint("1", text)
    }

    /// Dim, for less important text.
    pub fn dim(self, text: &str) -> String {
        self.paint("2", text)
    }

    /// The colour for a level: red for errors, yellow for warnings...
    ///
    /// Pad *before* painting: `{:<5}` counts the invisible escape bytes as
    /// characters, so padding a painted string misaligns the table.
    pub fn level(self, level: Level, text: &str) -> String {
        let code = match level {
            Level::Trace | Level::Debug => "2",
            Level::Info => "32",
            Level::Warn => "33",
            Level::Error => "31",
            Level::Fatal => "1;31",
        };
        self.paint(code, text)
    }
}

/// A one-line progress display on stderr, redrawn in place with `\r`.
///
/// It draws at most every 100 ms, because printing is slow and a progress
/// line that slows the work down defeats the point. Writes go to any
/// `W: Write` so the tests can capture them in a `Vec<u8>`.
#[derive(Debug)]
pub struct Progress<W: Write> {
    out: W,
    enabled: bool,
    label: String,
    total: Option<u64>,
    last_draw: Option<Instant>,
    drawn: bool,
}

/// How often the progress line may be redrawn.
const REDRAW_EVERY: Duration = Duration::from_millis(100);

impl<W: Write> Progress<W> {
    /// A progress line for reading `label`, which is `total` bytes long if
    /// known (files) or of unknown length (stdin). When `enabled` is false,
    /// every method does nothing.
    pub fn new(out: W, enabled: bool, label: impl Into<String>, total: Option<u64>) -> Self {
        Progress {
            out,
            enabled,
            label: label.into(),
            total,
            last_draw: None,
            drawn: false,
        }
    }

    /// Reports that `done` bytes have been read; redraws if it's time.
    pub fn update(&mut self, done: u64) {
        if !self.enabled {
            return;
        }
        let now = Instant::now();
        if self
            .last_draw
            .is_some_and(|last| now.duration_since(last) < REDRAW_EVERY)
        {
            return;
        }
        self.last_draw = Some(now);
        self.draw(done);
    }

    /// Draws the line now, whatever the time.
    pub fn draw(&mut self, done: u64) {
        if !self.enabled {
            return;
        }
        let text = match self.total {
            Some(total) if total > 0 => format!(
                "reading {}: {:>3}% ({} of {})",
                self.label,
                done.saturating_mul(100) / total,
                human_bytes(done),
                human_bytes(total)
            ),
            _ => format!("reading {}: {}", self.label, human_bytes(done)),
        };
        // `\r` goes back to the start of the line; `ESC[2K` clears it.
        // Progress is decoration: if stderr is closed, ignore the error.
        let _ = write!(self.out, "\r\x1b[2K{text}");
        let _ = self.out.flush();
        self.drawn = true;
    }

    /// Clears the line, so the real output starts on a clean line.
    pub fn finish(&mut self) {
        if self.enabled && self.drawn {
            let _ = write!(self.out, "\r\x1b[2K");
            let _ = self.out.flush();
            self.drawn = false;
        }
    }

    /// Gives back the writer (for tests).
    pub fn into_inner(self) -> W {
        self.out
    }
}

/// `1536` -> `"1.5 KiB"`.
///
/// ```
/// use capstone_cli_tool::human_bytes;
/// assert_eq!(human_bytes(512), "512 B");
/// assert_eq!(human_bytes(1536), "1.5 KiB");
/// assert_eq!(human_bytes(3 * 1024 * 1024), "3.0 MiB");
/// ```
pub fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["KiB", "MiB", "GiB", "TiB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64 / 1024.0;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_colour_needs_a_terminal_and_respects_no_color_and_dumb() {
        use ColorWhen::*;
        assert!(use_color(Auto, true, None, Some("xterm-256color")));
        assert!(use_color(Auto, true, Some(""), None)); // empty NO_COLOR = unset
        assert!(!use_color(Auto, false, None, None));
        assert!(!use_color(Auto, true, Some("yes"), None));
        assert!(!use_color(Auto, true, None, Some("dumb")));
        assert!(use_color(Always, false, Some("yes"), Some("dumb")));
        assert!(!use_color(Never, true, None, None));
    }

    #[test]
    fn color_when_parses_any_case() {
        assert_eq!("ALWAYS".parse(), Ok(ColorWhen::Always));
        assert!("sometimes".parse::<ColorWhen>().is_err());
    }

    #[test]
    fn palette_paints_only_when_enabled() {
        assert_eq!(Palette::new(false).level(Level::Error, "error"), "error");
        assert_eq!(
            Palette::new(true).level(Level::Error, "error"),
            "\x1b[31merror\x1b[0m"
        );
        assert_eq!(Palette::new(true).bold("x"), "\x1b[1mx\x1b[0m");
    }

    #[test]
    fn padding_a_painted_string_counts_the_invisible_bytes() {
        let painted = Palette::new(true).level(Level::Warn, "warn");
        // 4 visible letters, but 13 bytes: `{:<6}` adds no spaces at all.
        assert_eq!(format!("{painted:<6}|"), format!("{painted}|"));
        // Pad first, then paint: the spaces are inside the colour.
        let right = Palette::new(true).level(Level::Warn, &format!("{:<6}", "warn"));
        assert!(right.contains("warn  "));
    }

    #[test]
    fn progress_draws_in_place_and_clears_itself() {
        let mut p = Progress::new(Vec::new(), true, "app.log", Some(2048));
        p.update(1024);
        p.update(2048); // within 100 ms of the first draw: skipped
        p.finish();
        let out = String::from_utf8(p.into_inner()).unwrap();
        assert_eq!(
            out,
            "\r\x1b[2Kreading app.log:  50% (1.0 KiB of 2.0 KiB)\r\x1b[2K"
        );
    }

    #[test]
    fn disabled_progress_writes_nothing() {
        let mut p = Progress::new(Vec::new(), false, "-", None);
        p.draw(10);
        p.finish();
        assert!(p.into_inner().is_empty());
    }

    #[test]
    fn human_bytes_picks_a_unit() {
        assert_eq!(human_bytes(0), "0 B");
        assert_eq!(human_bytes(1024), "1.0 KiB");
        assert_eq!(human_bytes(5 * 1024 * 1024 * 1024), "5.0 GiB");
    }
}
