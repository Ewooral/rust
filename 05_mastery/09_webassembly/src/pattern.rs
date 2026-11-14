//! Life patterns in the "plaintext" (`.cells`) format.
//!
//! ```text
//! !Name: Glider
//! !A small spaceship that travels diagonally.
//! .O.
//! ..O
//! OOO
//! ```
//!
//! Lines starting with `!` are comments (`!Name:` gives the name), `O` is a
//! live cell and `.` a dead one. This is the text a user pastes into the web
//! page, so a mistake must come back as a clear message, not a crash: in
//! WebAssembly a panic aborts the whole module.

use std::fmt;

/// A parsed pattern: its size and the live cells, as (row, column).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    /// From a `!Name:` line, or empty.
    pub name: String,
    /// The widest row.
    pub width: u32,
    /// The number of rows.
    pub height: u32,
    /// The live cells.
    pub alive: Vec<(u32, u32)>,
}

/// What can be wrong with a pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternError {
    /// No rows at all (only comments, or nothing).
    Empty,
    /// A character that isn't `O` or `.`, with its 1-based position.
    BadChar {
        /// 1-based line in the text.
        line: usize,
        /// 1-based column, counted in characters.
        column: usize,
        /// What was found.
        found: char,
    },
    /// Bigger than [`MAX_SIDE`] in either direction.
    TooBig,
}

/// The largest pattern side accepted, to keep a pasted novel from eating
/// all the browser tab's memory.
pub const MAX_SIDE: u32 = 512;

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternError::Empty => f.write_str("the pattern has no rows"),
            PatternError::BadChar {
                line,
                column,
                found,
            } => write!(
                f,
                "line {line}, column {column}: {found:?} is not a cell (use O for alive, . for dead)"
            ),
            PatternError::TooBig => write!(f, "patterns can be at most {MAX_SIDE} cells across"),
        }
    }
}

impl std::error::Error for PatternError {}

/// Parses plaintext. Trailing spaces and `\r` are ignored.
///
/// # Errors
///
/// See [`PatternError`].
///
/// # Examples
///
/// ```
/// let blinker = webassembly::pattern::parse("!Name: Blinker\nOOO")?;
/// assert_eq!((blinker.name.as_str(), blinker.width, blinker.height), ("Blinker", 3, 1));
/// assert_eq!(blinker.alive, [(0, 0), (0, 1), (0, 2)]);
/// # Ok::<(), webassembly::pattern::PatternError>(())
/// ```
pub fn parse(text: &str) -> Result<Pattern, PatternError> {
    let mut name = String::new();
    let mut alive = Vec::new();
    let (mut width, mut height) = (0u32, 0u32);
    for (line_index, line) in text.lines().enumerate() {
        if let Some(comment) = line.strip_prefix('!') {
            if let Some(found) = comment.strip_prefix("Name:") {
                name = found.trim().to_string();
            }
            continue;
        }
        let row = line.trim_end();
        for (column, ch) in row.chars().enumerate() {
            match ch {
                'O' => alive.push((height, column as u32)),
                '.' => {}
                found => {
                    return Err(PatternError::BadChar {
                        line: line_index + 1,
                        column: column + 1,
                        found,
                    });
                }
            }
        }
        width = width.max(row.chars().count() as u32);
        height += 1;
        if width > MAX_SIDE || height > MAX_SIDE {
            return Err(PatternError::TooBig);
        }
    }
    if height == 0 {
        return Err(PatternError::Empty);
    }
    Ok(Pattern {
        name,
        width: width.max(1),
        height,
        alive,
    })
}

/// The patterns that come with the lesson, by name.
pub const BUILTINS: [(&str, &str); 5] = [
    ("glider", "!Name: Glider\n.O.\n..O\nOOO"),
    ("blinker", "!Name: Blinker\nOOO"),
    ("toad", "!Name: Toad\n.OOO\nOOO."),
    (
        "lwss",
        "!Name: Lightweight spaceship\n.O..O\nO....\nO...O\nOOOO.",
    ),
    (
        "pulsar",
        "!Name: Pulsar\n\
         ..OOO...OOO..\n\
         .............\n\
         O....O.O....O\n\
         O....O.O....O\n\
         O....O.O....O\n\
         ..OOO...OOO..\n\
         .............\n\
         ..OOO...OOO..\n\
         O....O.O....O\n\
         O....O.O....O\n\
         O....O.O....O\n\
         .............\n\
         ..OOO...OOO..",
    ),
];

/// A built-in pattern by name, such as `"glider"`.
pub fn builtin(name: &str) -> Option<Pattern> {
    let (_, text) = BUILTINS.iter().find(|(n, _)| *n == name)?;
    parse(text).ok()
}

/// The built-in pattern names.
pub fn builtin_names() -> impl Iterator<Item = &'static str> {
    BUILTINS.iter().map(|(name, _)| *name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_glider_with_comments() {
        let glider = builtin("glider").unwrap();
        assert_eq!(glider.name, "Glider");
        assert_eq!((glider.width, glider.height), (3, 3));
        assert_eq!(glider.alive, [(0, 1), (1, 2), (2, 0), (2, 1), (2, 2)]);
    }

    #[test]
    fn every_builtin_parses() {
        for name in builtin_names() {
            let pattern = builtin(name).unwrap_or_else(|| panic!("{name}"));
            assert!(!pattern.alive.is_empty(), "{name}");
        }
        assert_eq!(builtin("pulsar").unwrap().alive.len(), 48);
        assert!(builtin("unicorn").is_none());
    }

    #[test]
    fn short_rows_and_windows_line_endings_are_fine() {
        let p = parse("!Name: Ragged\r\nO\r\n..O  \r\n").unwrap();
        assert_eq!((p.width, p.height), (3, 2));
        assert_eq!(p.alive, [(0, 0), (1, 2)]);
    }

    #[test]
    fn errors_point_at_the_character() {
        assert_eq!(
            parse("!Name: Oops\n.O.\n.X."),
            Err(PatternError::BadChar {
                line: 3,
                column: 2,
                found: 'X'
            })
        );
        assert_eq!(
            parse("!only a comment").unwrap_err().to_string(),
            "the pattern has no rows"
        );
        assert_eq!(parse(""), Err(PatternError::Empty));
        let wide = "O".repeat(MAX_SIDE as usize + 1);
        assert_eq!(parse(&wide), Err(PatternError::TooBig));
    }

    #[test]
    fn columns_count_characters_not_bytes() {
        let err = parse("..é").unwrap_err();
        assert_eq!(
            err.to_string(),
            "line 1, column 3: 'é' is not a cell (use O for alive, . for dead)"
        );
    }
}
