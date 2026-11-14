//! Tasks: `tokio::spawn`, `JoinHandle`, panics and `JoinError`, `abort`,
//! detaching, `JoinSet`, and `spawn_blocking` for CPU-heavy work.
//! Run: cargo run -p tokio_runtime --example tasks

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::task::JoinSet;
use tokio::time::sleep;

fn count_primes(below: u64) -> usize {
    (2..below)
        .filter(|n| (2..).take_while(|d| d * d <= *n).all(|d| n % d != 0))
        .count()
}

#[tokio::main]
async fn main() {
    // 1. spawn returns a JoinHandle; awaiting it gives Result<T, JoinError>.
    let handle = tokio::spawn(async { 6 * 7 });
    println!("1. the task returned {:?}", handle.await);

    // 2. A panic stays inside its task: the runtime and main carry on.
    let handle = tokio::spawn(async {
        if true {
            panic!("oops in a task");
        }
        0
    });
    let err = handle.await.unwrap_err();
    println!("2. panicked task -> is_panic = {}", err.is_panic());

    // 3. abort() cancels a task at its next .await.
    let handle = tokio::spawn(async {
        sleep(Duration::from_secs(60)).await;
        "never"
    });
    handle.abort();
    let err = handle.await.unwrap_err();
    println!("3. aborted task -> is_cancelled = {}", err.is_cancelled());

    // 4. Dropping a JoinHandle does NOT stop the task: it is detached.
    let finished = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&finished);
    drop(tokio::spawn(async move {
        sleep(Duration::from_millis(20)).await;
        flag.store(true, Ordering::SeqCst);
    }));
    sleep(Duration::from_millis(60)).await;
    println!(
        "4. detached task still finished: {}",
        finished.load(Ordering::SeqCst)
    );

    // 5. JoinSet: a group of tasks; join_next gives results as they FINISH.
    let mut set = JoinSet::new();
    for (name, ms) in [("slow", 60), ("fast", 10), ("medium", 30)] {
        set.spawn(async move {
            sleep(Duration::from_millis(ms)).await;
            name
        });
    }
    let mut order = Vec::new();
    while let Some(result) = set.join_next().await {
        order.push(result.unwrap());
    }
    println!("5. JoinSet finishing order: {order:?}");
    // (Dropping a JoinSet aborts every task still in it.)

    // 6. spawn_blocking: CPU-heavy work on a separate thread pool, so the
    //    async worker threads stay free. A ticker keeps ticking meanwhile.
    let ticker = tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_millis(5));
        loop {
            interval.tick().await; // runs until aborted
        }
    });
    let primes = tokio::task::spawn_blocking(|| count_primes(200_000))
        .await
        .unwrap();
    println!(
        "6. spawn_blocking counted {primes} primes; the ticker kept running: {}",
        !ticker.is_finished()
    );
    ticker.abort();
}
