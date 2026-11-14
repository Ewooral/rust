//! Solution to exercise 1: make it compile (E0373).
//!
//! The helper thread borrows `chores`, but a spawned thread may keep running
//! after the function that started it has returned, so the compiler can't
//! prove the borrow stays valid. Fix it with ONE keyword, so the thread
//! takes ownership of `chores`.
//! Then try to print `chores` in `main` after the spawn: what error do you
//! get now, and why?
//!
//! Run: cargo run -p fearless_concurrency --features solutions --bin sol01_fix_missing_move

use std::thread;

fn main() {
    let chores = vec!["wash up", "feed the cat", "water the plants"];

    // `move`: the closure takes ownership of `chores`, so the thread can
    // never outlive the data it uses.
    let handle = thread::spawn(move || {
        for chore in &chores {
            println!("helper: {chore}");
        }
        chores.len()
    });

    let done = handle.join().expect("the helper thread panicked");
    println!("main: the helper did {done} chores");
}
