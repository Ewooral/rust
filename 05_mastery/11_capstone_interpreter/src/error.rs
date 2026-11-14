//! Where something is in the source ([`Span`]) and what went wrong there
//! ([`Error`]). Every stage reports problems with the same type.

use std::fmt;

/// A position in the source: 1-based line and column (in characters).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn new(line: usize, col: usize) -> Self {
        Span { line, col }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.col)
    }
}

/// Which stage found the problem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// The lexer or the parser: the program isn't valid Kite.
    Syntax,
    /// The evaluator: valid Kite that went wrong while running.
    Runtime,
}

/// An error with a position, from any stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Span,
    /// True when the parser ran out of tokens: the REPL then asks for
    /// another line instead of reporting the error.
    pub at_end: bool,
}

impl Error {
    pub fn syntax(message: impl Into<String>, span: Span) -> Self {
        Error {
            kind: ErrorKind::Syntax,
            message: message.into(),
            span,
            at_end: false,
        }
    }

    pub fn runtime(message: impl Into<String>, span: Span) -> Self {
        Error {
            kind: ErrorKind::Runtime,
            message: message.into(),
            span,
            at_end: false,
        }
    }

    /// The error with the offending source line and a `^` under the
    /// column, the way rustc shows its errors:
    ///
    /// ```text
    /// runtime error: division by zero
    ///  --> line 2, column 9
    ///   |
    /// 2 | print(1 / 0);
    ///   |         ^
    /// ```
    pub fn render(&self, source: &str) -> String {
        let mut text = format!("{self}");
        let Some(code) = source.lines().nth(self.span.line.saturating_sub(1)) else {
            return text;
        };
        let number = self.span.line.to_string();
        let gutter = " ".repeat(number.len());
        // Keep tabs as tabs so the caret lines up under them.
        let pad: String = code
            .chars()
            .take(self.span.col.saturating_sub(1))
            .map(|c| if c == '\t' { '\t' } else { ' ' })
            .collect();
        text.push_str(&format!(
            "\n{gutter}--> {}\n{gutter} |\n{number} | {code}\n{gutter} | {pad}^",
            self.span
        ));
        text
    }
}

impl fmt::Display for Error {
    /// `"syntax error at line 1, column 5: expected ';'"`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self.kind {
            ErrorKind::Syntax => "syntax error",
            ErrorKind::Runtime => "runtime error",
        };
        write!(f, "{kind} at {}: {}", self.span, self.message)
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_names_the_stage_and_position() {
        let e = Error::runtime("division by zero", Span::new(2, 9));
        assert_eq!(
            e.to_string(),
            "runtime error at line 2, column 9: division by zero"
        );
    }

    #[test]
    fn render_points_at_the_column() {
        let source = "let x = 1;\nprint(1 / 0);\n";
        let e = Error::runtime("division by zero", Span::new(2, 9));
        assert_eq!(
            e.render(source),
            "runtime error at line 2, column 9: division by zero\n --> line 2, column 9\n  |\n2 | print(1 / 0);\n  |         ^"
        );
    }

    #[test]
    fn render_without_the_line_is_just_the_message() {
        let e = Error::syntax("unexpected end of input", Span::new(9, 1));
        assert_eq!(e.render("one line"), e.to_string());
    }
}
