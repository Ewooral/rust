//! Exercise 3: make it compile (E0502).
//!
//! `record` mutably borrows `log` for as long as the closure is still going
//! to be used. Printing `log` in between is a second borrow at the same
//! time. Reorder the lines (no clones, no new variables) so every call to
//! `record` happens before `log` is printed.
//!
//! Run: cargo run -p iterators_and_closures --features exercises --bin ex03_fix_mutable_capture

fn main() {
    let mut log: Vec<String> = Vec::new();
    let mut record = |event: &str| log.push(event.to_string());

    record("started");
    println!("so far: {log:?}");
    record("finished");
}
