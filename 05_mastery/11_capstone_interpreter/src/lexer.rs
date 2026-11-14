//! Stage 1, the lexer: source text -> a list of tokens, each with its span.

use std::iter::Peekable;
use std::str::Chars;

use crate::error::{Error, Span};
use crate::token::{Token, TokenKind};

/// Splits `source` into tokens. The last token is always `Eof`.
///
/// ```
/// use capstone_interpreter::{TokenKind, lex};
/// let tokens = lex("let x = 1;")?;
/// let kinds: Vec<_> = tokens.into_iter().map(|t| t.kind).collect();
/// assert_eq!(kinds, [
///     TokenKind::Let, TokenKind::Ident("x".into()), TokenKind::Equal,
///     TokenKind::Number(1.0), TokenKind::Semicolon, TokenKind::Eof,
/// ]);
/// # Ok::<(), capstone_interpreter::Error>(())
/// ```
///
/// # Errors
///
/// A syntax error for a character Kite doesn't use, or a string that
/// isn't closed or has an unknown escape.
pub fn lex(source: &str) -> Result<Vec<Token>, Error> {
    Lexer::new(source).run()
}

/// Walks the source one character at a time, tracking line and column.
struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
    col: usize,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Lexer {
            chars: source.chars().peekable(),
            line: 1,
            col: 1,
            tokens: Vec::new(),
        }
    }

    /// Takes the next character and moves the position on.
    fn bump(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    /// Takes the next character only if it is `expected`.
    fn bump_if(&mut self, expected: char) -> bool {
        if self.chars.peek() == Some(&expected) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn run(mut self) -> Result<Vec<Token>, Error> {
        loop {
            let start = Span::new(self.line, self.col);
            let Some(c) = self.bump() else {
                self.tokens.push(Token {
                    kind: TokenKind::Eof,
                    span: start,
                });
                return Ok(self.tokens);
            };
            let kind = match c {
                ' ' | '\t' | '\r' | '\n' => continue,
                '/' if self.bump_if('/') => {
                    // A comment: skip to the end of the line.
                    while self.chars.peek().is_some_and(|&c| c != '\n') {
                        self.bump();
                    }
                    continue;
                }
                '(' => TokenKind::LParen,
                ')' => TokenKind::RParen,
                '{' => TokenKind::LBrace,
                '}' => TokenKind::RBrace,
                ',' => TokenKind::Comma,
                ';' => TokenKind::Semicolon,
                '+' => TokenKind::Plus,
                '-' => TokenKind::Minus,
                '*' => TokenKind::Star,
                '/' => TokenKind::Slash,
                // Two-character operators: look one character ahead.
                '!' if self.bump_if('=') => TokenKind::BangEqual,
                '!' => TokenKind::Bang,
                '=' if self.bump_if('=') => TokenKind::EqualEqual,
                '=' => TokenKind::Equal,
                '<' if self.bump_if('=') => TokenKind::LessEqual,
                '<' => TokenKind::Less,
                '>' if self.bump_if('=') => TokenKind::GreaterEqual,
                '>' => TokenKind::Greater,
                '"' => self.string(start)?,
                c if c.is_ascii_digit() => self.number(c),
                c if c.is_alphabetic() || c == '_' => self.word(c),
                other => {
                    return Err(Error::syntax(
                        format!("unexpected character {other:?}"),
                        start,
                    ));
                }
            };
            self.tokens.push(Token { kind, span: start });
        }
    }

    /// `123` or `3.25`.
    fn number(&mut self, first: char) -> TokenKind {
        let mut text = String::from(first);
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() {
                text.push(c);
                self.bump();
            } else if c == '.' && !text.contains('.') {
                // Only a decimal point if a digit follows: `1.5`, not `1.`.
                let mut ahead = self.chars.clone();
                ahead.next();
                if !ahead.peek().is_some_and(char::is_ascii_digit) {
                    break;
                }
                text.push(c);
                self.bump();
            } else {
                break;
            }
        }
        // Only digits and at most one '.', so this can't fail.
        TokenKind::Number(text.parse().expect("digits form a valid number"))
    }

    /// A name or a keyword.
    fn word(&mut self, first: char) -> TokenKind {
        let mut text = String::from(first);
        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || c == '_' {
                text.push(c);
                self.bump();
            } else {
                break;
            }
        }
        TokenKind::keyword(&text).unwrap_or(TokenKind::Ident(text))
    }

    /// The rest of a string after its opening `"`.
    fn string(&mut self, start: Span) -> Result<TokenKind, Error> {
        let mut text = String::new();
        loop {
            let here = Span::new(self.line, self.col);
            match self.bump() {
                None | Some('\n') => {
                    return Err(Error::syntax("unterminated string", start));
                }
                Some('"') => return Ok(TokenKind::Str(text)),
                Some('\\') => match self.bump() {
                    Some('n') => text.push('\n'),
                    Some('t') => text.push('\t'),
                    Some('"') => text.push('"'),
                    Some('\\') => text.push('\\'),
                    Some(other) => {
                        return Err(Error::syntax(
                            format!("unknown escape \\{other} in a string"),
                            here,
                        ));
                    }
                    None => return Err(Error::syntax("unterminated string", start)),
                },
                Some(c) => text.push(c),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use TokenKind::*;

    fn kinds(source: &str) -> Vec<TokenKind> {
        lex(source).unwrap().into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn punctuation_and_two_character_operators() {
        assert_eq!(
            kinds("( ) { } , ; + - * / ! != = == < <= > >="),
            [
                LParen,
                RParen,
                LBrace,
                RBrace,
                Comma,
                Semicolon,
                Plus,
                Minus,
                Star,
                Slash,
                Bang,
                BangEqual,
                Equal,
                EqualEqual,
                Less,
                LessEqual,
                Greater,
                GreaterEqual,
                Eof
            ]
        );
    }

    #[test]
    fn numbers_names_and_keywords() {
        assert_eq!(
            kinds("let total_2 = 3.25 + 10;"),
            [
                Let,
                Ident("total_2".into()),
                Equal,
                Number(3.25),
                Plus,
                Number(10.0),
                Semicolon,
                Eof
            ]
        );
        assert_eq!(
            kinds("fn if else while return true false nil and or"),
            [Fn, If, Else, While, Return, True, False, Nil, And, Or, Eof]
        );
    }

    #[test]
    fn strings_with_escapes() {
        assert_eq!(
            kinds(r#""say \"hi\"\n""#),
            [Str("say \"hi\"\n".into()), Eof]
        );
        assert_eq!(kinds("\"héllo\""), [Str("héllo".into()), Eof]);
    }

    #[test]
    fn comments_and_whitespace_are_skipped() {
        assert_eq!(
            kinds("1 // one\n// a whole line\n2"),
            [Number(1.0), Number(2.0), Eof]
        );
    }

    #[test]
    fn spans_are_line_and_column_of_the_first_character() {
        let tokens = lex("let x\n  = 10;").unwrap();
        let spans: Vec<_> = tokens.iter().map(|t| (t.span.line, t.span.col)).collect();
        assert_eq!(spans, [(1, 1), (1, 5), (2, 3), (2, 5), (2, 7), (2, 8)]);
    }

    #[test]
    fn a_dot_needs_a_digit_after_it() {
        let err = lex("1.").unwrap_err();
        assert_eq!(
            err.to_string(),
            "syntax error at line 1, column 2: unexpected character '.'"
        );
    }

    #[test]
    fn errors_have_positions() {
        assert_eq!(
            lex("let x = 1 # 2;").unwrap_err().to_string(),
            "syntax error at line 1, column 11: unexpected character '#'"
        );
        assert_eq!(
            lex("print(\"oops);").unwrap_err().to_string(),
            "syntax error at line 1, column 7: unterminated string"
        );
        assert_eq!(
            lex(r#""a\qb""#).unwrap_err().to_string(),
            "syntax error at line 1, column 3: unknown escape \\q in a string"
        );
    }
}
