//! SOLUTION to exercise 2: fix the borrow conflict.
//!
//! This does not compile (E0502). Make it compile so that it prints
//! `first score: 10` and then `all scores: [10, 20, 30, 40]`.
//! Think about WHY Rust refuses: what could `push` do to `first`?
//!
//! Run: cargo run -p ownership --features solutions --bin sol02_fix_borrow_conflict

fn main() {
    let mut scores = vec![10, 20, 30];
    let first = &scores[0];
    println!("first score: {first}"); // last use of the borrow
    scores.push(40); // now nothing borrows `scores`
    println!("all scores: {scores:?}");
}
