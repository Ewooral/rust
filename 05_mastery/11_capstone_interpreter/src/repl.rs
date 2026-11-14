//! The REPL ("read, evaluate, print, loop"), without the terminal: lines
//! go in, events come out. `main.rs` connects it to stdin and stdout, and
//! the tests feed it strings.

use std::io::Write;

use crate::interpreter::Interpreter;
use crate::value::Value;

/// What happened after a line was fed in.
#[derive(Debug, PartialEq)]
pub enum Event {
    /// The input so far is unfinished (an open `{` or `(`): send another line.
    NeedMore,
    /// It ran. If it was an expression with a value other than nil, here it is.
    Done(Option<Value>),
    /// It failed: the rendered error, with the line and a `^`.
    Error(String),
}

/// A REPL session. Variables and functions persist between entries.
pub struct Repl<W: Write> {
    interpreter: Interpreter<W>,
    pending: String,
}

impl<W: Write> Repl<W> {
    pub fn new(out: W) -> Self {
        Repl {
            interpreter: Interpreter::new(out),
            pending: String::new(),
        }
    }

    /// Whether a previous line is waiting to be continued (show `...`).
    pub fn is_continuing(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Feeds one line of input.
    ///
    /// ```
    /// use capstone_interpreter::{Event, Repl, Value};
    /// let mut repl = Repl::new(Vec::new());
    /// assert_eq!(repl.feed("fn double(x) {"), Event::NeedMore);
    /// assert_eq!(repl.feed("  return x * 2; }"), Event::Done(None));
    /// assert_eq!(repl.feed("double(21)"), Event::Done(Some(Value::Number(42.0))));
    /// ```
    pub fn feed(&mut self, line: &str) -> Event {
        let blank = line.trim().is_empty();
        if blank && self.pending.is_empty() {
            return Event::Done(None);
        }
        self.pending.push_str(line);
        self.pending.push('\n');

        let source = std::mem::take(&mut self.pending);
        match self.interpreter.run_repl(&source) {
            Ok(value) => Event::Done(value.filter(|v| *v != Value::Nil)),
            // Ran out of input mid-way: wait for more, unless the user
            // pressed Enter on an empty line to give up.
            Err(e) if e.at_end && !blank => {
                self.pending = source;
                Event::NeedMore
            }
            Err(e) => Event::Error(e.render(&source)),
        }
    }

    /// The interpreter's output (what `print` wrote).
    pub fn output(&self) -> &W {
        self.interpreter.output()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expressions_show_their_value_statements_do_not() {
        let mut repl = Repl::new(Vec::new());
        assert_eq!(repl.feed("let x = 20;"), Event::Done(None));
        assert_eq!(repl.feed("x + 1"), Event::Done(Some(Value::Number(21.0))));
        assert_eq!(repl.feed("x + 1;"), Event::Done(Some(Value::Number(21.0))));
        assert_eq!(repl.feed("print(\"hi\")"), Event::Done(None)); // nil isn't shown
        assert_eq!(repl.output(), b"hi\n");
    }

    #[test]
    fn unfinished_input_asks_for_more() {
        let mut repl = Repl::new(Vec::new());
        assert_eq!(repl.feed("fn add(a, b) {"), Event::NeedMore);
        assert!(repl.is_continuing());
        assert_eq!(repl.feed("return a + b;"), Event::NeedMore);
        assert_eq!(repl.feed("}"), Event::Done(None));
        assert!(!repl.is_continuing());
        assert_eq!(
            repl.feed("add(2, 3)"),
            Event::Done(Some(Value::Number(5.0)))
        );
    }

    #[test]
    fn errors_are_rendered_and_the_session_carries_on() {
        let mut repl = Repl::new(Vec::new());
        let Event::Error(text) = repl.feed("1 / 0") else {
            panic!("expected an error")
        };
        assert_eq!(
            text,
            "runtime error at line 1, column 3: division by zero\n --> line 1, column 3\n  |\n1 | 1 / 0\n  |   ^"
        );
        assert_eq!(repl.feed("2 * 3"), Event::Done(Some(Value::Number(6.0))));
    }

    #[test]
    fn a_blank_line_gives_up_on_unfinished_input() {
        let mut repl = Repl::new(Vec::new());
        assert_eq!(repl.feed("(1 +"), Event::NeedMore);
        assert!(matches!(repl.feed(""), Event::Error(_)));
        assert!(!repl.is_continuing());
    }
}
