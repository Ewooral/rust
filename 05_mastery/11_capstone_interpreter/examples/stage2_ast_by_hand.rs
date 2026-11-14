//! Stage 2a: the AST is a tree of enums, and every link to a child is a
//! `Box` (or a `Vec`). Here we build `1 + 2 * 3` by hand, then let the
//! parser build it, and check they're the same tree.
//! Run: cargo run -p capstone_interpreter --example stage2_ast_by_hand

use std::mem::size_of;

use capstone_interpreter::ast::BinaryOp;
use capstone_interpreter::{Expr, ExprKind, Span, parse_expr};

fn number(n: f64, col: usize) -> Expr {
    Expr::new(ExprKind::Number(n), Span::new(1, col))
}

fn binary(op: BinaryOp, left: Expr, right: Expr, col: usize) -> Expr {
    Expr::new(
        ExprKind::Binary {
            op,
            left: Box::new(left), // the child lives on the heap; we hold a pointer
            right: Box::new(right),
        },
        Span::new(1, col),
    )
}

/// A tiny evaluator for this tree: recursion follows the shape of the data.
fn eval(expr: &Expr) -> f64 {
    match &expr.kind {
        ExprKind::Number(n) => *n,
        ExprKind::Binary { op, left, right } => {
            let (a, b) = (eval(left), eval(right));
            match op {
                BinaryOp::Add => a + b,
                BinaryOp::Multiply => a * b,
                _ => unimplemented!("only + and * in this example"),
            }
        }
        _ => unimplemented!("only numbers in this example"),
    }
}

fn main() {
    //        +            1 + 2 * 3
    //       / \
    //      1   *
    //         / \
    //        2   3
    let by_hand = binary(
        BinaryOp::Add,
        number(1.0, 1),
        binary(BinaryOp::Multiply, number(2.0, 5), number(3.0, 9), 7),
        3,
    );
    let parsed = parse_expr("1 + 2 * 3").unwrap();

    println!("by hand: {by_hand}");
    println!("parsed:  {parsed}");
    println!("same tree (spans included): {}", by_hand == parsed);
    println!("value: {}", eval(&by_hand));

    // Why Box? Without it, an Expr would contain two whole Exprs, each
    // containing two more... an infinite size (exercise 1: E0072). A Box is
    // one pointer, so the size is known:
    println!(
        "\nsize_of::<Expr>() = {} bytes, size_of::<Box<Expr>>() = {} bytes",
        size_of::<Expr>(),
        size_of::<Box<Expr>>()
    );
}
