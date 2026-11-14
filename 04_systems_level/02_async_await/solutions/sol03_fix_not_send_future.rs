//! Solution to exercise 3: make it compile ("future cannot be sent between threads
//! safely"; this error has no code).
//!
//! `tokio::spawn` may move a task to another worker thread at any `.await`,
//! so the whole future must be `Send`. Each task here keeps a
//! `std::sync::MutexGuard` alive ACROSS an `.await`, and a guard must be
//! unlocked on the thread that locked it, so it isn't `Send`.
//! Fix it WITHOUT changing the type of `visits`: finish with the lock
//! (copy the number out) before the `.await`, so the guard is dropped first.
//! (The other fix is `tokio::sync::Mutex`, whose guard may cross `.await`s.)
//!
//! Run: cargo run -p async_await --features solutions --bin sol03_fix_not_send_future

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let visits = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();

    for id in 1..=3 {
        let visits = Arc::clone(&visits);
        handles.push(tokio::spawn(async move {
            // Lock, update, copy the number out, and unlock (the guard is
            // dropped at the end of this block), all BEFORE the `.await`.
            let count = {
                let mut guard = visits.lock().unwrap();
                *guard += 1;
                *guard
            };
            sleep(Duration::from_millis(10)).await; // pretend to save it somewhere
            println!("task {id} recorded visit number {count}");
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
    println!("total visits: {}", *visits.lock().unwrap());
}
