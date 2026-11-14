//! Book 17.6: putting it all together: futures, tasks and threads.
//!
//! - A FUTURE is a value that can be polled until it's done.
//! - A TASK is a future the runtime schedules on its own (`tokio::spawn`).
//! - A THREAD is the operating system's unit: the runtime's worker threads
//!   run many tasks each.
//!
//! Run: cargo run -p async_await --example futures_tasks_threads

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;

#[tokio::main] // the multi-threaded runtime: one worker thread per core
async fn main() {
    // 1. Tasks are CHEAP: a sleeping task uses no thread at all, just its
    // small state machine. Start 10,000 tasks that each sleep 50 ms: they
    // all sleep AT THE SAME TIME, on a handful of worker threads.
    let start = Instant::now();
    let handles: Vec<_> = (0..10_000)
        .map(|i| {
            tokio::spawn(async move {
                sleep(Duration::from_millis(50)).await;
                i
            })
        })
        .collect();
    let mut total = 0u64;
    for handle in handles {
        total += handle.await.unwrap();
    }
    println!(
        "1. 10,000 sleeping tasks finished in {:?}, not 10,000 x 50 ms (sum {total})",
        start.elapsed()
    );
    // The same with OS threads would need 10,000 threads, each with its own
    // stack (reserved memory) and kernel bookkeeping. For lots of WAITING,
    // tasks are the right tool.

    // 2. Which threads ran our tasks? Tokio's workers STEAL tasks from each
    // other, and a task may resume on a different thread after each `.await`.
    // That's why `tokio::spawn` requires `Send` futures.
    let seen = Arc::new(Mutex::new(HashSet::new()));
    let handles: Vec<_> = (0..50)
        .map(|_| {
            let seen = Arc::clone(&seen);
            tokio::spawn(async move {
                seen.lock().unwrap().insert(thread::current().id());
                sleep(Duration::from_millis(1)).await;
                seen.lock().unwrap().insert(thread::current().id());
            })
        })
        .collect();
    for handle in handles {
        handle.await.unwrap();
    }
    println!(
        "2. 50 tasks ran on {} different worker threads",
        seen.lock().unwrap().len()
    );

    // 3. The Book's example: a THREAD produces, async code consumes. A
    // Tokio channel's `send` is not async, so plain threads can use it too.
    let (tx, mut rx) = mpsc::unbounded_channel();
    thread::spawn(move || {
        for i in 1..=5 {
            tx.send(i).unwrap();
            thread::sleep(Duration::from_millis(5));
        }
    });
    let mut received = Vec::new();
    while let Some(message) = rx.recv().await {
        received.push(message);
    }
    println!("3. async code received from a thread: {received:?}");

    // 4. Blocking or CPU-heavy work inside a task stalls a worker thread and
    // every task waiting on it. `spawn_blocking` moves it to a separate pool
    // of threads meant for blocking, and gives you a future for the result.
    let heavy = tokio::task::spawn_blocking(|| {
        thread::sleep(Duration::from_millis(30)); // pretend: a big file or a long sum
        (1..=1_000_000u64).sum::<u64>()
    });
    let light = async {
        sleep(Duration::from_millis(5)).await;
        "light work done while the heavy work ran"
    };
    let (heavy, light) = tokio::join!(heavy, light);
    println!("4. {light}; heavy result = {}", heavy.unwrap());

    println!("\nRule of thumb:");
    println!("  lots of waiting (network, timers) -> async tasks");
    println!("  heavy computation                 -> threads, rayon, or spawn_blocking");
    println!(
        "  both                              -> async, plus spawn_blocking for the heavy parts"
    );
}
