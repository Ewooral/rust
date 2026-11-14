//! Solution to exercise 1: make it compile.
//!
//! `mut` is right here: the score is one value that changes during the game.
//! Shadowing would also compile, but it reads as "a new score", and it
//! couldn't be done inside a loop that keeps adding to the same score.
//!
//! Run: cargo run -p variables_and_data_types --features solutions --bin sol01_fix_immutable_assign

fn main() {
    let mut score = 0;
    println!("kick-off: {score}");
    score += 10;
    println!("after a goal: {score}");
}
