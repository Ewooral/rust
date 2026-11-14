//! Exercise 1: make it compile.
//!
//! The score goes up after a goal, but the program doesn't compile (E0384).
//! Fix it by adding ONE word. Then think: could shadowing
//! (`let score = score + 10;`) fix it too? Which one says what you mean?
//!
//! Run: cargo run -p variables_and_data_types --features exercises --bin ex01_fix_immutable_assign

fn main() {
    let score = 0;
    println!("kick-off: {score}");
    score += 10;
    println!("after a goal: {score}");
}
