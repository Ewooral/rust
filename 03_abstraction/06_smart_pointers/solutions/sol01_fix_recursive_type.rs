//! Solution to exercise 1: make it compile (E0072).
//!
//! Each inner `Expr` is now a `Box<Expr>`: a pointer of known size (8 bytes
//! on 64-bit) to an expression on the heap, so `Expr` has a fixed size.
//! `eval` didn't change: `a` is a `&Box<Expr>`, and deref coercion turns it
//! into the `&Expr` that `eval` wants.
//!
//! Run: cargo run -p smart_pointers --features solutions --bin sol01_fix_recursive_type

#[derive(Debug)]
enum Expr {
    Num(i64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
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
    let expr = Mul(
        Box::new(Add(Box::new(Num(2)), Box::new(Num(3)))),
        Box::new(Neg(Box::new(Num(4)))),
    );
    println!("{expr:?} = {}", eval(&expr));
    println!("size of Expr: {} bytes", std::mem::size_of::<Expr>());
}
