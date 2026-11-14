//! Stage 2b: Pratt parsing, traced. A 40-line Pratt parser for numbers and
//! `+ - * /` prints every decision it makes, then the real parser shows
//! the grouping it chose for some trickier expressions.
//! Run: cargo run -p capstone_interpreter --example stage3_pratt_parser

use capstone_interpreter::parse_expr;

/// (left, right) binding powers. Higher binds tighter.
fn power(op: char) -> Option<(u8, u8)> {
    match op {
        '+' | '-' => Some((1, 2)),
        '*' | '/' => Some((3, 4)),
        _ => None,
    }
}

/// Parses tokens[*pos..] while operators bind at least `min`; returns an
/// S-expression string. `depth` only indents the trace.
fn expr(tokens: &[char], pos: &mut usize, min: u8, depth: usize) -> String {
    let pad = "  ".repeat(depth);
    let mut left = tokens[*pos].to_string(); // a one-digit number
    *pos += 1;
    println!("{pad}start with {left} (min power {min})");
    while let Some(&op) = tokens.get(*pos) {
        let (l, r) = power(op).expect("an operator");
        if l < min {
            println!("{pad}'{op}' has power {l} < {min}: stop, give {left} back");
            break;
        }
        println!("{pad}'{op}' has power {l} >= {min}: take it, parse its right side");
        *pos += 1;
        let right = expr(tokens, pos, r, depth + 1);
        left = format!("({op} {left} {right})");
        println!("{pad}now: {left}");
    }
    left
}

fn trace(source: &str) {
    let tokens: Vec<char> = source.chars().filter(|c| !c.is_whitespace()).collect();
    println!("=== {source}");
    let result = expr(&tokens, &mut 0, 0, 0);
    println!("result: {result}\n");
}

fn main() {
    trace("1 + 2 * 3"); // * binds tighter: it grabs the 2
    trace("1 * 2 + 3"); // + is weaker: the * finishes first
    trace("8 - 3 - 2"); // left-associative: (8 - 3) - 2

    println!("=== the real parser (src/parser.rs)");
    for source in [
        "-2 * 3",
        "1 + 2 < 4 == true",
        "a or b and c",
        "a = b = 1 + 2",
        "-f(x)(y)",
        "!done and n > 0",
    ] {
        println!("{source:<20} => {}", parse_expr(source).unwrap());
    }
}
