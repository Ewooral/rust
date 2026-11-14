//! Tokens: the "words" of a Kite program.

use std::fmt;

use crate::error::Span;

/// What kind of word a token is. Literals carry their value.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Punctuation
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Semicolon,
    // Operators
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
    // Literals and names
    Number(f64),
    Str(String),
    Ident(String),
    // Keywords
    Let,
    Fn,
    If,
    Else,
    While,
    Return,
    True,
    False,
    Nil,
    And,
    Or,
    /// The end of the input. The parser always finds one, so it never has
    /// to check for "no more tokens".
    Eof,
}

impl TokenKind {
    /// The keyword for an identifier-shaped word, if it is one.
    pub fn keyword(word: &str) -> Option<TokenKind> {
        Some(match word {
            "let" => TokenKind::Let,
            "fn" => TokenKind::Fn,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "return" => TokenKind::Return,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "nil" => TokenKind::Nil,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            _ => return None,
        })
    }
}

impl fmt::Display for TokenKind {
    /// How the token looks in source, for error messages: `'{'`, `number 3`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            TokenKind::LParen => "'('",
            TokenKind::RParen => "')'",
            TokenKind::LBrace => "'{'",
            TokenKind::RBrace => "'}'",
            TokenKind::Comma => "','",
            TokenKind::Semicolon => "';'",
            TokenKind::Plus => "'+'",
            TokenKind::Minus => "'-'",
            TokenKind::Star => "'*'",
            TokenKind::Slash => "'/'",
            TokenKind::Bang => "'!'",
            TokenKind::BangEqual => "'!='",
            TokenKind::Equal => "'='",
            TokenKind::EqualEqual => "'=='",
            TokenKind::Less => "'<'",
            TokenKind::LessEqual => "'<='",
            TokenKind::Greater => "'>'",
            TokenKind::GreaterEqual => "'>='",
            TokenKind::Number(n) => return write!(f, "number {n}"),
            TokenKind::Str(s) => return write!(f, "string {s:?}"),
            TokenKind::Ident(name) => return write!(f, "name '{name}'"),
            TokenKind::Let => "'let'",
            TokenKind::Fn => "'fn'",
            TokenKind::If => "'if'",
            TokenKind::Else => "'else'",
            TokenKind::While => "'while'",
            TokenKind::Return => "'return'",
            TokenKind::True => "'true'",
            TokenKind::False => "'false'",
            TokenKind::Nil => "'nil'",
            TokenKind::And => "'and'",
            TokenKind::Or => "'or'",
            TokenKind::Eof => "the end of the input",
        };
        f.write_str(text)
    }
}

/// A token and where it starts.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
