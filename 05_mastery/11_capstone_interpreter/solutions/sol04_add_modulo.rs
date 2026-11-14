//! Solution to exercise 4: extend the language with a new operator, `%` (remainder).
//!
//! The calculator from exercise 3, with its precedence fixed. Add `%` the
//! way every new operator is added to a real interpreter, stage by stage:
//!
//! 1. Lexer: `lex` rejects '%' as an unexpected character. Make it an `Op`.
//! 2. Parser: give `%` a binding power. It belongs with `*` and `/`:
//!    `1 + 7 % 4` is `1 + (7 % 4)`, and `10 % 4 % 3` is `(10 % 4) % 3`.
//! 3. Evaluator: compute it with Rust's `%` on f64 (so `-7 % 3` is -1, the
//!    sign of the left side), and report `"modulo by zero"` instead of
//!    returning NaN.
//!
//! Three small edits, one per stage: '%' in the lexer's operator list, in
//! the `*`/`/` row of `infix_power`, and a new arm in `eval`. Because `eval`
//! matches on the operator, forgetting step 3 would hit `unreachable!()`.
//!
//! Then try the same three steps on the real Kite (see the README's project
//! section) and rewrite programs/fizzbuzz.kite with `n % 3 == 0`.
//!
//! Test: cargo test -p capstone_interpreter --features solutions --bin sol04_add_modulo

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Token {
    Num(f64),
    Op(char),
    LParen,
    RParen,
}

fn lex(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            ' ' => {
                chars.next();
            }
            '0'..='9' | '.' => {
                let mut text = String::new();
                while let Some(&d) = chars.peek().filter(|d| d.is_ascii_digit() || **d == '.') {
                    text.push(d);
                    chars.next();
                }
                let n = text.parse().map_err(|_| format!("bad number {text:?}"))?;
                tokens.push(Token::Num(n));
            }
            '+' | '-' | '*' | '/' | '%' => {
                tokens.push(Token::Op(c));
                chars.next();
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            other => return Err(format!("unexpected character {other:?}")),
        }
    }
    Ok(tokens)
}

#[derive(Debug)]
enum Expr {
    Num(f64),
    Neg(Box<Expr>),
    Bin(char, Box<Expr>, Box<Expr>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Num(n) => write!(f, "{n}"),
            Expr::Neg(inner) => write!(f, "(- {inner})"),
            Expr::Bin(op, left, right) => write!(f, "({op} {left} {right})"),
        }
    }
}

/// (left power, right power) for each infix operator. Higher binds tighter.
/// For a LEFT-associative operator, the right power is one MORE than the
/// left; one LESS makes it right-associative.
fn infix_power(op: char) -> (u8, u8) {
    match op {
        '+' | '-' => (5, 6),
        '*' | '/' | '%' => (7, 8),
        _ => unreachable!("the lexer only makes + - * / %"),
    }
}

/// Unary minus binds tighter than every infix operator.
const PREFIX_POWER: u8 = 9;

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).copied();
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
            other => return Err(format!("expected a number, found {other:?}")),
        };
        while let Some(&Token::Op(op)) = self.tokens.get(self.pos) {
            let (left_power, right_power) = infix_power(op);
            if left_power < min_power {
                break;
            }
            self.pos += 1;
            let right = self.expr(right_power)?;
            left = Expr::Bin(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }
}

fn parse(source: &str) -> Result<Expr, String> {
    let mut parser = Parser {
        tokens: lex(source)?,
        pos: 0,
    };
    let expr = parser.expr(0)?;
    match parser.tokens.get(parser.pos) {
        None => Ok(expr),
        Some(token) => Err(format!("unexpected {token:?}")),
    }
}

fn eval(expr: &Expr) -> Result<f64, String> {
    Ok(match expr {
        Expr::Num(n) => *n,
        Expr::Neg(inner) => -eval(inner)?,
        Expr::Bin(op, left, right) => {
            let (a, b) = (eval(left)?, eval(right)?);
            match op {
                '+' => a + b,
                '-' => a - b,
                '*' => a * b,
                '/' if b == 0.0 => return Err("division by zero".into()),
                '/' => a / b,
                '%' if b == 0.0 => return Err("modulo by zero".into()),
                '%' => a % b,
                _ => unreachable!(),
            }
        }
    })
}

fn main() {
    for source in ["7 % 3", "1 + 7 % 4", "10 % 4 % 3", "-7 % 3", "5 % 0"] {
        match parse(source) {
            Ok(expr) => println!(
                "{source:<14} parses as {:<22} = {:?}",
                expr.to_string(),
                eval(&expr)
            ),
            Err(e) => println!("{source:<14} error: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calc(source: &str) -> Result<f64, String> {
        eval(&parse(source)?)
    }

    #[test]
    fn the_lexer_accepts_percent() {
        assert_eq!(
            lex("7 % 3").unwrap(),
            [Token::Num(7.0), Token::Op('%'), Token::Num(3.0)]
        );
    }

    #[test]
    fn remainder_values() {
        assert_eq!(calc("7 % 3"), Ok(1.0));
        assert_eq!(calc("9 % 3"), Ok(0.0));
        assert_eq!(calc("7.5 % 2"), Ok(1.5));
        assert_eq!(calc("-7 % 3"), Ok(-1.0));
    }

    #[test]
    fn same_precedence_as_multiplication_and_left_associative() {
        assert_eq!(parse("1 + 7 % 4").unwrap().to_string(), "(+ 1 (% 7 4))");
        assert_eq!(parse("10 % 4 % 3").unwrap().to_string(), "(% (% 10 4) 3)");
        assert_eq!(parse("2 * 9 % 4").unwrap().to_string(), "(% (* 2 9) 4)");
        assert_eq!(calc("1 + 7 % 4"), Ok(4.0));
        assert_eq!(calc("10 % 4 % 3"), Ok(2.0));
    }

    #[test]
    fn modulo_by_zero_is_an_error() {
        assert_eq!(calc("5 % 0"), Err("modulo by zero".to_string()));
        assert_eq!(calc("5 % (2 - 2)"), Err("modulo by zero".to_string()));
    }
}
