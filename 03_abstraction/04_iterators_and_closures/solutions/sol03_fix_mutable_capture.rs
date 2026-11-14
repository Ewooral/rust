//! Solution to exercise 3: make it compile (E0502).
//!
//! `record` mutably borrows `log` for as long as the closure is still going
//! to be used. Printing `log` in between is a second borrow at the same
//! time. Reorder the lines (no clones, no new variables) so every call to
//! `record` happens before `log` is printed.
//!
//! Run: cargo run -p iterators_and_closures --features solutions --bin sol03_fix_mutable_capture

fn main() {
    let mut log: Vec<String> = Vec::new();
    let mut record = |event: &str| log.push(event.to_string());

    record("started");
    record("finished");
    // The closure is not used any more, so its mutable borrow has ended.
    println!("so far: {log:?}");
}
