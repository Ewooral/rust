//! Solution to exercise 2: make it compile ("future cannot be sent between threads
//! safely"; this error has no code).
//!
//! `tokio::spawn` hands the task to the multi-threaded runtime: it may run,
//! and continue after every `.await`, on ANY worker thread, so everything
//! the task owns must be `Send`. `Rc` isn't: its reference count is a
//! plain number, and two threads changing it at once would corrupt it.
//!
//! Fix it by sharing the list with the thread-safe version of `Rc`.
//! (The other way out, for code that really must use `Rc`, is to run the
//! tasks on ONE thread with `tokio::task::LocalSet` and `spawn_local`.)
//!
//! Run: cargo run -p tokio_runtime --features solutions --bin sol02_fix_rc_across_await

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    // Arc: like Rc, but its count is atomic, so it is Send and Sync.
    let names = Arc::new(vec![String::from("ada"), String::from("grace")]);
    let mut handles = Vec::new();

    for i in 0..names.len() {
        let names = Arc::clone(&names);
        handles.push(tokio::spawn(async move {
            sleep(Duration::from_millis(10)).await; // pretend to look something up
            format!("task {i} greets {}", names[i])
        }));
    }

    for handle in handles {
        println!("{}", handle.await.unwrap());
    }
}
