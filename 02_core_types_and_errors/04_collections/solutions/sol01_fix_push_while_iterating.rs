//! Solution to exercise 1: make it compile (E0502).
//!
//! For every score above 50, add a bonus score of double its value to the end.
//! The loop is reading `scores` while trying to push into it. Fix it by
//! collecting the new values into a separate Vec first, then adding them all
//! after the loop (`extend` adds many at once).
//!
//! Expected output: [40, 60, 80, 120, 160]
//!
//! Run: cargo run -p collections --features solutions --bin sol01_fix_push_while_iterating

fn main() {
    let mut scores = vec![40, 60, 80];
    let mut bonus = Vec::new();
    for s in &scores {
        if *s > 50 {
            bonus.push(s * 2);
        }
    }
    // The loop's borrow of `scores` has ended, so we may change it now.
    scores.extend(bonus);
    println!("{scores:?}");
}
