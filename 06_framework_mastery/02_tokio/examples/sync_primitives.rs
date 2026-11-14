//! Sync primitives from `tokio::sync`: `Mutex` (vs std's), `RwLock`,
//! `Semaphore`, `Notify` and `OnceCell`.
//! Run: cargo run -p tokio_runtime --example sync_primitives
//!
//! The clock is paused (`start_paused`, from Tokio's `test-util` feature,
//! enabled for examples and tests in Cargo.toml): it jumps straight to the
//! next timer whenever every task is waiting, so the times printed are exact.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::sync::{Mutex, Notify, OnceCell, RwLock, Semaphore};
use tokio::task::JoinSet;
use tokio::time::{Instant, sleep};

#[tokio::main(flavor = "current_thread", start_paused = true)]
async fn main() {
    // 1. tokio::sync::Mutex: its guard may be held across an `.await`
    //    (a std MutexGuard must not be). Use it only when you need that;
    //    a std Mutex is faster for short, await-free critical sections.
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut set = JoinSet::new();
    for id in 1..=3 {
        let log = Arc::clone(&log);
        set.spawn(async move {
            let mut guard = log.lock().await; // waits without blocking the thread
            sleep(Duration::from_millis(10)).await; // still holding the lock
            guard.push(id);
        });
    }
    set.join_all().await;
    println!("1. Mutex: every task wrote in turn: {:?}", log.lock().await);

    // 2. RwLock: many readers at once, or one writer.
    let config = Arc::new(RwLock::new(String::from("v1")));
    let start = Instant::now();
    let mut set = JoinSet::new();
    for _ in 0..3 {
        let config = Arc::clone(&config);
        set.spawn(async move {
            let value = config.read().await;
            sleep(Duration::from_millis(100)).await; // three readers overlap
            value.clone()
        });
    }
    let seen = set.join_all().await;
    *config.write().await = String::from("v2");
    println!(
        "2. RwLock: 3 readers saw {seen:?} in {} ms total; now {}",
        start.elapsed().as_millis(),
        config.read().await
    );

    // 3. Semaphore: at most N at a time (here 3 of 10 "downloads").
    let permits = Arc::new(Semaphore::new(3));
    let running = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let start = Instant::now();
    let mut set = JoinSet::new();
    for _ in 0..10 {
        let (permits, running, peak) = (permits.clone(), running.clone(), peak.clone());
        set.spawn(async move {
            let _permit = permits.acquire().await.unwrap(); // released on drop
            let now = running.fetch_add(1, Ordering::SeqCst) + 1;
            peak.fetch_max(now, Ordering::SeqCst);
            sleep(Duration::from_millis(100)).await;
            running.fetch_sub(1, Ordering::SeqCst);
        });
    }
    set.join_all().await;
    println!(
        "3. Semaphore(3): 10 jobs of 100 ms, at most {} at once, {} ms total",
        peak.load(Ordering::SeqCst),
        start.elapsed().as_millis()
    );

    // 4. Notify: one task waits for a signal, another sends it. No data.
    let ready = Arc::new(Notify::new());
    let waiter = {
        let ready = Arc::clone(&ready);
        tokio::spawn(async move {
            ready.notified().await;
            "woken up"
        })
    };
    sleep(Duration::from_millis(50)).await;
    ready.notify_one(); // a permit is stored if nobody is waiting yet
    println!("4. Notify: the waiter says {:?}", waiter.await.unwrap());

    // 5. OnceCell: async initialisation that runs exactly once, however
    //    many tasks ask for the value at the same time.
    static SETTINGS: OnceCell<String> = OnceCell::const_new();
    static LOADS: AtomicUsize = AtomicUsize::new(0);
    let mut set = JoinSet::new();
    for _ in 0..5 {
        set.spawn(async {
            SETTINGS
                .get_or_init(|| async {
                    LOADS.fetch_add(1, Ordering::SeqCst);
                    sleep(Duration::from_millis(30)).await; // "read a file"
                    String::from("dark mode")
                })
                .await
                .clone()
        });
    }
    let values = set.join_all().await;
    println!(
        "5. OnceCell: 5 tasks got {:?}, loaded {} time(s)",
        values[0],
        LOADS.load(Ordering::SeqCst)
    );
}
