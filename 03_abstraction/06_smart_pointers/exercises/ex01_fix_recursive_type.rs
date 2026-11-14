//! Exercise 1: make it compile (E0072).
//!
//! An arithmetic expression is a tree: `Add` holds two smaller expressions.
//! But an `Expr` that holds an `Expr` directly would need infinite space,
//! so the compiler can't work out its size.
//!
//! Put every inner expression in a `Box`: change the enum, and wrap the
//! inner expressions in `main` with `Box::new(...)`. `eval` needs no
//! change at all. Why not? (Hint: deref coercion, Book 15.2.)
//!
//! Run: cargo run -p smart_pointers --features exercises --bin ex01_fix_recursive_type

#[derive(Debug)]
enum Expr {
    Num(i64),
    Add(Expr, Expr),
    Mul(Expr, Expr),
    Neg(Expr),
}

use Expr::{Add, Mul, Neg, Num};

fn eval(expr: &Expr) -> i64 {
    match expr {
        Num(n) => *n,
        Add(a, b) => eval(a) + eval(b),
        Mul(a, b) => eval(a) * eval(b),
        Neg(inner) => -eval(inner),
    }
}

fn main() {
    // (2 + 3) * -4
    let expr = Mul(Add(Num(2), Num(3)), Neg(Num(4)));
    println!("{expr:?} = {}", eval(&expr));
}
