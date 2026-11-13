//! Exercise 1: make it compile (E0502).
//!
//! For every score above 50, add a bonus score of double its value to the end.
//! The loop is reading `scores` while trying to push into it. Fix it by
//! collecting the new values into a separate Vec first, then adding them all
//! after the loop (`extend` adds many at once).
//!
//! Expected output: [40, 60, 80, 120, 160]
//!
//! Run: cargo run -p collections --features exercises --bin ex01_fix_push_while_iterating

fn main() {
    let mut scores = vec![40, 60, 80];
    for s in &scores {
        if *s > 50 {
            scores.push(s * 2);
        }
    }
    println!("{scores:?}");
}
