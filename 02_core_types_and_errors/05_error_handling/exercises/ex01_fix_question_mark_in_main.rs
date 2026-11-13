//! Exercise 1: make it compile (E0277).
//!
//! `?` needs somewhere to send the error, but `main` returns nothing. Change
//! ONLY the signature of `main` (and its last line) so `?` works.
//! Then change "42" to "forty-two" and run it again: what does Rust print,
//! and what exit code does the program end with?
//!
//! Run: cargo run -p error_handling --features exercises --bin ex01_fix_question_mark_in_main

fn main() {
    let answer: i32 = "42".parse()?;
    println!("the answer is {answer}");
}
