//! Solution to exercise 3: fix two precedence bugs in a Pratt parser.
//!
//! This is a tiny calculator built exactly like Kite's expression parser
//! (src/parser.rs), cut down to numbers, `+ - * /`, unary `-` and brackets.
//! It compiles and runs, but gets answers wrong:
//!
//!   1 + 2 * 3    gives 9,  should be 7   (* must bind tighter than +)
//!   10 - 3 - 2   gives 9,  should be 5   (- must be LEFT-associative)
//!
//! Both bugs are in the binding-power table, `infix_power`. Read how
//! `expr` uses the two numbers, run the tests, and fix the table. Don't
//! touch anything else. The tests print the parsed shape, `(+ 1 (* 2 3))`,
//! which shows you exactly how the parser grouped things.
//!
//! The fixes:
//! - `*` and `/` had LOWER powers (3, 4) than `+` (5, 6), so `+` grabbed
//!   operands first. Give them higher ones: (7, 8).
//! - `-` had (5, 4): a right power below the left one makes `10 - 3 - 2`
//!   parse as `10 - (3 - 2)`. Same powers as `+`, (5, 6), make the second
//!   `-` stop (5 < 6) and hand `3` back, so it groups as `(10 - 3) - 2`.
//!
//! `PREFIX_POWER` (9) stays above every infix power, so `-2 * 3` is `(-2) * 3`.
//!
//! Test: cargo test -p capstone_interpreter --features solutions --bin sol03_fix_precedence

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
            '+' | '-' | '*' | '/' => {
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
        '*' | '/' => (7, 8),
        _ => unreachable!("the lexer only makes + - * /"),
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
                _ => unreachable!(),
            }
        }
    })
}

fn main() {
    for source in [
        "1 + 2 * 3",
        "10 - 3 - 2",
        "2 * 3 + 4",
        "8 / 4 / 2",
        "-(1 + 2) * 3",
    ] {
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

    fn check(source: &str, shape: &str, value: f64) {
        let expr = parse(source).unwrap();
        assert_eq!(expr.to_string(), shape, "the shape of {source}");
        assert_eq!(eval(&expr).unwrap(), value, "the value of {source}");
    }

    #[test]
    fn multiplication_binds_tighter_than_addition() {
        check("1 + 2 * 3", "(+ 1 (* 2 3))", 7.0);
        check("2 * 3 + 4", "(+ (* 2 3) 4)", 10.0);
        check("1 - 6 / 2", "(- 1 (/ 6 2))", -2.0);
    }

    #[test]
    fn same_power_operators_group_from_the_left() {
        check("10 - 3 - 2", "(- (- 10 3) 2)", 5.0);
        check("8 / 4 / 2", "(/ (/ 8 4) 2)", 1.0);
        check("1 - 2 + 3", "(+ (- 1 2) 3)", 2.0);
    }

    #[test]
    fn brackets_and_unary_minus_still_work() {
        check("(1 + 2) * 3", "(* (+ 1 2) 3)", 9.0);
        check("-2 * 3", "(* (- 2) 3)", -6.0);
        check("-(1 + 2) * 3", "(* (- (+ 1 2)) 3)", -9.0);
    }
}
