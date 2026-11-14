//! The runtime: the multi-thread and current-thread flavours, building one
//! with `Builder`, and what blocking code does to a runtime.
//! Run: cargo run -p tokio_runtime --example runtime
//!
//! `#[tokio::main]` is a macro that writes the `Builder` code for you:
//!
//!   #[tokio::main]                              -> Builder::new_multi_thread().enable_all()
//!   #[tokio::main(worker_threads = 2)]          -> ...with .worker_threads(2)
//!   #[tokio::main(flavor = "current_thread")]   -> Builder::new_current_thread().enable_all()
//!
//! and then calls `.build().unwrap().block_on(async { ...your main... })`.
//! This example builds the runtimes by hand to see them clearly.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::{Builder, Handle};
use tokio::task::JoinSet;

/// Spawns `n` tasks that each block their thread for 20 ms, and returns
/// the names of the threads they ran on.
async fn where_do_tasks_run(n: usize) -> BTreeSet<String> {
    let mut set = JoinSet::new();
    for _ in 0..n {
        set.spawn(async {
            thread::sleep(Duration::from_millis(20)); // busy, on purpose
            thread::current().name().unwrap_or("unnamed").to_string()
        });
    }
    set.join_all().await.into_iter().collect()
}

fn main() {
    // 1. current_thread: ONE thread (the one calling block_on) runs every task.
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    let names = rt.block_on(where_do_tasks_run(8));
    println!("current_thread: 8 tasks ran on {names:?}");

    // 2. multi_thread: a pool of worker threads; idle workers steal tasks
    //    from busy ones. `enable_all` turns on the timer and the I/O driver.
    static NEXT: AtomicUsize = AtomicUsize::new(1);
    let rt = Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name_fn(|| format!("worker-{}", NEXT.fetch_add(1, Ordering::Relaxed)))
        .enable_all()
        .build()
        .unwrap();
    let (workers, names) = rt.block_on(async {
        let workers = Handle::current().metrics().num_workers();
        (workers, where_do_tasks_run(8).await)
    });
    println!(
        "multi_thread: {workers} workers; 8 tasks ran on {} different threads",
        names.len()
    );
    drop(rt); // shuts the pool down

    // 3. Blocking code freezes a current_thread runtime. Task A sleeps 10 ms
    //    the async way; task B blocks the thread for 100 ms. A's timer is
    //    due after 10 ms, but nobody can run A until B lets go.
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async {
        let start = Instant::now();
        let a = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            start.elapsed()
        });
        let b = tokio::spawn(async {
            thread::sleep(Duration::from_millis(100)); // blocking!
        });
        b.await.unwrap();
        let a_done = a.await.unwrap().as_millis() / 50 * 50;
        println!("blocked runtime: a 10 ms timer finished after about {a_done} ms");
    });

    // 4. The fix: hand blocking work to spawn_blocking's separate thread pool.
    rt.block_on(async {
        let start = Instant::now();
        let a = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            start.elapsed()
        });
        let b = tokio::task::spawn_blocking(|| thread::sleep(Duration::from_millis(100)));
        let a_done = a.await.unwrap().as_millis();
        b.await.unwrap();
        println!(
            "with spawn_blocking: the timer finished after under 50 ms: {}",
            a_done < 50
        );
    });
}
