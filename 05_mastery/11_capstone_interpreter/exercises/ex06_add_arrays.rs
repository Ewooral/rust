//! Exercise 6: extend the language with arrays.
//!
//! The calculator from exercises 3 and 4 now has a `Value` type with two
//! kinds of value, numbers and arrays, and its lexer already knows `[`,
//! `]` and `,`. Add the two pieces of syntax:
//!
//! - Array literals (a PREFIX: `[` can start an expression):
//!   `[1, 2 + 3, 4]`, `[]`, `[[1, 2], [3]]`
//! - Indexing (a POSTFIX, like a call in Kite, binding tightest of all):
//!   `xs[1]`, `[[1, 2], [3, 4]][1][0]`, `-[5][0]` (the minus applies last)
//!
//! Implement:
//! - `Parser::array_literal`: after the `[`, zero or more expressions
//!   separated by `,`, then `]` (else `Err("expected ']'")`).
//! - `Parser::index`: after the `[`, one expression, then `]`.
//! - `index`: the value at a position, with these errors:
//!   `"can only index arrays, not a number"`,
//!   `"the index must be a whole number, not 0.5"`,
//!   `"index 3 is out of range for an array of length 3"`.
//!
//! Test: cargo test -p capstone_interpreter --features exercises --bin ex06_add_arrays

#![allow(unused_variables, dead_code)]

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Token {
    Num(f64),
    Op(char),
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
}

fn lex(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    while let Some(&c) = chars.peek() {
        let token = match c {
            ' ' => {
                chars.next();
                continue;
            }
            '0'..='9' | '.' => {
                let mut text = String::new();
                while let Some(&d) = chars.peek().filter(|d| d.is_ascii_digit() || **d == '.') {
                    text.push(d);
                    chars.next();
                }
                let n = text.parse().map_err(|_| format!("bad number {text:?}"))?;
                tokens.push(Token::Num(n));
                continue;
            }
            '+' | '-' | '*' | '/' => Token::Op(c),
            '(' => Token::LParen,
            ')' => Token::RParen,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            ',' => Token::Comma,
            other => return Err(format!("unexpected character {other:?}")),
        };
        tokens.push(token);
        chars.next();
    }
    Ok(tokens)
}

#[derive(Debug)]
enum Expr {
    Num(f64),
    Neg(Box<Expr>),
    Bin(char, Box<Expr>, Box<Expr>),
    /// `[a, b, c]`
    Array(Vec<Expr>),
    /// `target[index]`
    Index(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Num(f64),
    Array(Vec<Value>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Num(n) => write!(f, "{n}"),
            Value::Array(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
        }
    }
}

/// The kind of value, with its article: "a number", "an array".
fn kind(value: &Value) -> &'static str {
    match value {
        Value::Num(_) => "a number",
        Value::Array(_) => "an array",
    }
}

fn infix_power(op: char) -> (u8, u8) {
    match op {
        '+' | '-' => (5, 6),
        '*' | '/' => (7, 8),
        _ => unreachable!("the lexer only makes + - * /"),
    }
}

const PREFIX_POWER: u8 = 9;
/// `xs[i]` binds tighter than anything else.
const POSTFIX_POWER: u8 = 11;

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.peek();
        self.pos += 1;
        token
    }

    fn expr(&mut self, min_power: u8) -> Result<Expr, String> {
        let mut left = match self.next() {
            Some(Token::Num(n)) => Expr::Num(n),
            Some(Token::Op('-')) => Expr::Neg(Box::new(self.expr(PREFIX_POWER)?)),
            Some(Token::LParen) => {
                let inner = self.expr(0)?;
                if self.next() != Some(Token::RParen) {
                    return Err("expected ')'".into());
                }
                inner
            }
            Some(Token::LBracket) => self.array_literal()?,
            other => return Err(format!("expected a value, found {other:?}")),
        };
        loop {
            match self.peek() {
                Some(Token::LBracket) => {
                    if POSTFIX_POWER < min_power {
                        break;
                    }
                    self.pos += 1;
                    left = self.index(left)?;
                }
                Some(Token::Op(op)) => {
                    let (left_power, right_power) = infix_power(op);
                    if left_power < min_power {
                        break;
                    }
                    self.pos += 1;
                    let right = self.expr(right_power)?;
                    left = Expr::Bin(op, Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// After `[`: `a, b, c]` (or just `]`).
    fn array_literal(&mut self) -> Result<Expr, String> {
        todo!()
    }

    /// After `target[`: `index]`.
    fn index(&mut self, target: Expr) -> Result<Expr, String> {
        todo!()
    }
}

fn parse(source: &str) -> Result<Expr, String> {
    let mut parser = Parser {
        tokens: lex(source)?,
        pos: 0,
    };
    let expr = parser.expr(0)?;
    match parser.peek() {
        None => Ok(expr),
        Some(token) => Err(format!("unexpected {token:?}")),
    }
}

/// `target[position]`
fn index(target: Value, position: Value) -> Result<Value, String> {
    todo!()
}

fn eval(expr: &Expr) -> Result<Value, String> {
    Ok(match expr {
        Expr::Num(n) => Value::Num(*n),
        Expr::Neg(inner) => match eval(inner)? {
            Value::Num(n) => Value::Num(-n),
            other => return Err(format!("cannot negate {}", kind(&other))),
        },
        Expr::Bin(op, left, right) => match (eval(left)?, eval(right)?) {
            (Value::Num(a), Value::Num(b)) => Value::Num(match op {
                '+' => a + b,
                '-' => a - b,
                '*' => a * b,
                '/' if b == 0.0 => return Err("division by zero".into()),
                '/' => a / b,
                _ => unreachable!(),
            }),
            (a, b) => {
                return Err(format!(
                    "cannot use '{op}' on {} and {}",
                    kind(&a),
                    kind(&b)
                ));
            }
        },
        Expr::Array(items) => {
            let mut values = Vec::with_capacity(items.len());
            for item in items {
                values.push(eval(item)?);
            }
            Value::Array(values)
        }
        Expr::Index(target, position) => index(eval(target)?, eval(position)?)?,
    })
}

fn calc(source: &str) -> Result<Value, String> {
    eval(&parse(source)?)
}

fn main() {
    for source in [
        "[1, 2 + 3, 4]",
        "[1, 2 + 3, 4][1]",
        "[[1, 2], [3, 4]][1][0]",
        "-[5][0] * 2",
        "[]",
        "[1, 2][2]",
        "5[0]",
    ] {
        match calc(source) {
            Ok(value) => println!("{source:<24} = {value}"),
            Err(e) => println!("{source:<24} error: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn num(n: f64) -> Value {
        Value::Num(n)
    }

    #[test]
    fn array_literals() {
        assert_eq!(calc("[]"), Ok(Value::Array(vec![])));
        assert_eq!(
            calc("[1, 2 + 3, 4]"),
            Ok(Value::Array(vec![num(1.0), num(5.0), num(4.0)]))
        );
        assert_eq!(calc("[[1], []]").unwrap().to_string(), "[[1], []]");
        assert_eq!(calc("[1, 2"), Err("expected ']'".to_string()));
    }

    #[test]
    fn indexing() {
        assert_eq!(calc("[10, 20, 30][0]"), Ok(num(10.0)));
        assert_eq!(calc("[10, 20, 30][1 + 1]"), Ok(num(30.0)));
        assert_eq!(calc("[[1, 2], [3, 4]][1][0]"), Ok(num(3.0)));
        assert_eq!(calc("[1, 2][1] * 10 + 1"), Ok(num(21.0)));
    }

    #[test]
    fn indexing_binds_tighter_than_unary_minus() {
        assert_eq!(calc("-[5][0]"), Ok(num(-5.0)));
        assert_eq!(calc("-[5][0] * 2"), Ok(num(-10.0)));
    }

    #[test]
    fn index_errors() {
        assert_eq!(
            calc("[1, 2, 3][3]"),
            Err("index 3 is out of range for an array of length 3".to_string())
        );
        assert_eq!(
            calc("5[0]"),
            Err("can only index arrays, not a number".to_string())
        );
        assert_eq!(
            calc("[1, 2][0.5]"),
            Err("the index must be a whole number, not 0.5".to_string())
        );
        assert_eq!(
            calc("[1] + 1"),
            Err("cannot use '+' on an array and a number".to_string())
        );
    }
}
