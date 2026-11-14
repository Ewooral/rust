//! Solution to exercise 1: make it compile (E0072).
//!
//! A tiny AST for arithmetic. An expression can contain other expressions,
//! so the enum refers to itself. But Rust must know the SIZE of every type,
//! and an `Expr` that directly contained two more `Expr`s, which each
//! contained two more... would be infinitely big.
//!
//! The fix: `Box<Expr>` for every recursive field. A Box is a pointer, so
//! its size is fixed, and the children live on the heap.
//!
//! Answer: `eval` needed no change because `Box<Expr>` dereferences to
//! `Expr` (the `Deref` trait, lesson 03/06). `eval(a)` with `a: &Box<Expr>`
//! is turned into `eval(&**a)` automatically ("deref coercion").
//!
//! Run: cargo run -p capstone_interpreter --features solutions --bin sol01_fix_recursive_enum

#[derive(Debug)]
enum Expr {
    Number(f64),
    Add(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Negate(Box<Expr>),
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
        Box::new(Expr::Add(
            Box::new(Expr::Number(1.0)),
            Box::new(Expr::Number(2.0)),
        )),
        Box::new(Expr::Negate(Box::new(Expr::Number(3.0)))),
    );
    println!("{expr:?}");
    println!("= {}", eval(&expr));
    println!(
        "size_of::<Expr>() = {} bytes, whatever the tree's size",
        std::mem::size_of::<Expr>()
    );
}
