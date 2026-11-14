//! Solution to exercise 6: an RPN calculator, with a pattern in every place you can.
//!
//! Reverse Polish Notation writes the operator AFTER its two numbers:
//! "3 4 + 2 *" means (3 + 4) * 2 = 14. It needs no brackets, just a stack.
//!
//! 1. `apply`: ONE `match` on the tuple `(op, a, b)`. Dividing by zero is
//!    `Err(DivideByZero)`: put a literal `0` inside the tuple pattern.
//! 2. `calculate`: `for` each whitespace-separated token, `match` it:
//!    - "+", "-", "*" or "/" (one arm, with `|`): pop the top TWO numbers
//!      with one `let (Some(b), Some(a)) = (stack.pop(), stack.pop()) else`
//!      (else `Err(NotEnoughNumbers)`), then push `apply(token, a, b)?`.
//!    - anything else: parse it with `let Ok(n) = ... else` (else
//!      `Err(BadToken(token))`) and push it. ("-3" is a number, not "-".)
//! 3. At the end, `match stack.as_slice()`: exactly one number is the answer,
//!    none is `Err(Empty)`, more is `Err(LeftOver(how_many))`.
//!
//! Test: cargo test -p patterns_and_matching --features solutions --bin sol06_rpn_calculator

#[derive(Debug, PartialEq)]
enum RpnError {
    BadToken(String),
    NotEnoughNumbers,
    DivideByZero,
    Empty,
    LeftOver(usize),
}

fn apply(op: &str, a: i64, b: i64) -> Result<i64, RpnError> {
    match (op, a, b) {
        ("+", a, b) => Ok(a + b),
        ("-", a, b) => Ok(a - b),
        ("*", a, b) => Ok(a * b),
        ("/", _, 0) => Err(RpnError::DivideByZero), // literal 0 in the tuple
        ("/", a, b) => Ok(a / b),
        (other, ..) => Err(RpnError::BadToken(other.to_string())),
    }
}

fn calculate(input: &str) -> Result<i64, RpnError> {
    let mut stack: Vec<i64> = Vec::new();

    for token in input.split_whitespace() {
        match token {
            "+" | "-" | "*" | "/" => {
                // A tuple pattern in let-else: both pops must be Some. The
                // first pop is the TOP of the stack, which is the RIGHT operand.
                let (Some(b), Some(a)) = (stack.pop(), stack.pop()) else {
                    return Err(RpnError::NotEnoughNumbers);
                };
                stack.push(apply(token, a, b)?);
            }
            other => {
                let Ok(n) = other.parse::<i64>() else {
                    return Err(RpnError::BadToken(other.to_string()));
                };
                stack.push(n);
            }
        }
    }

    match stack.as_slice() {
        [answer] => Ok(*answer),
        [] => Err(RpnError::Empty),
        left_over => Err(RpnError::LeftOver(left_over.len())),
    }
}

fn main() {
    for input in ["3 4 + 2 *", "5 1 2 + 4 * + 3 -", "4 0 /", "1 +", "1 2"] {
        println!("{input:>18} = {:?}", calculate(input));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates() {
        assert_eq!(calculate("42"), Ok(42));
        assert_eq!(calculate("3 4 + 2 *"), Ok(14));
        assert_eq!(calculate("5 1 2 + 4 * + 3 -"), Ok(14));
        assert_eq!(calculate("-3 4 *"), Ok(-12));
        assert_eq!(calculate("10 3 /"), Ok(3));
    }

    #[test]
    fn reports_bad_steps() {
        assert_eq!(calculate("1 +"), Err(RpnError::NotEnoughNumbers));
        assert_eq!(calculate("4 0 /"), Err(RpnError::DivideByZero));
        assert_eq!(calculate("2 x +"), Err(RpnError::BadToken("x".into())));
        assert_eq!(apply("%", 1, 2), Err(RpnError::BadToken("%".into())));
    }

    #[test]
    fn checks_what_is_left() {
        assert_eq!(calculate(""), Err(RpnError::Empty));
        assert_eq!(calculate("   "), Err(RpnError::Empty));
        assert_eq!(calculate("1 2 3 +"), Err(RpnError::LeftOver(2)));
    }
}
