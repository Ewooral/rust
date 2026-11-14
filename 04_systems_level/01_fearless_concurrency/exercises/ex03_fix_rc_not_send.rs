//! Exercise 3: make it compile (E0277).
//!
//! Five players add their points to one shared scoreboard. `Rc` keeps its
//! reference count in a plain (non-atomic) number, so two threads cloning
//! or dropping it at the same moment could corrupt the count: `Rc` is not
//! `Send`. Swap it for its thread-safe twin (change the `use` line and the
//! two `Rc::` calls). The total should be 150.
//!
//! Run: cargo run -p fearless_concurrency --features exercises --bin ex03_fix_rc_not_send

use std::rc::Rc;
use std::sync::Mutex;
use std::thread;

fn main() {
    let scoreboard = Rc::new(Mutex::new(0));
    let mut handles = vec![];

    for points in [10, 20, 30, 40, 50] {
        let scoreboard = Rc::clone(&scoreboard);
        let handle = thread::spawn(move || {
            let mut total = scoreboard.lock().unwrap();
            *total += points;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("final score: {}", *scoreboard.lock().unwrap());
}
