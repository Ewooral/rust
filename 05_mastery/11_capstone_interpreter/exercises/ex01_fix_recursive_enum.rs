//! Exercise 1: make it compile (E0072).
//!
//! A tiny AST for arithmetic. An expression can contain other expressions,
//! so the enum refers to itself. But Rust must know the SIZE of every type,
//! and an `Expr` that directly contains two more `Expr`s, which each contain
//! two more... would be infinitely big.
//!
//! Read the compiler's `help:` line, then put each recursive field in a
//! `Box` (a pointer to a value on the heap: always 8 bytes on a 64-bit
//! machine). The constructors in `main` need `Box::new(...)` too, and
//! `eval` needs no change at all: why? (Answer in the solution.)
//!
//! Run: cargo run -p capstone_interpreter --features exercises --bin ex01_fix_recursive_enum

#[derive(Debug)]
enum Expr {
    Number(f64),
    Add(Expr, Expr),
    Multiply(Expr, Expr),
    Negate(Expr),
}

fn eval(expr: &Expr) -> f64 {
    match expr {
        Expr::Number(n) => *n,
        Expr::Add(a, b) => eval(a) + eval(b),
        Expr::Multiply(a, b) => eval(a) * eval(b),
        Expr::Negate(inner) => -eval(inner),
    }
}

fn main() {
    // (1 + 2) * -3
    let expr = Expr::Multiply(
        Expr::Add(Expr::Number(1.0), Expr::Number(2.0)),
        Expr::Negate(Expr::Number(3.0)),
    );
    println!("{expr:?}");
    println!("= {}", eval(&expr));
}
