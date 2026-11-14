//! Beyond the Book: Tokio in practice: `#[tokio::main]`, timeouts,
//! `tokio::select!`, cancellation, `JoinSet`, and cancellation safety.
//!
//! Run: cargo run -p async_await --example tokio_in_practice

use std::pin::pin;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinSet;
use tokio::time::{interval, sleep, timeout};

// `#[tokio::main]` rewrites `async fn main` into roughly:
//     fn main() {
//         tokio::runtime::Builder::new_multi_thread().enable_all().build()
//             .unwrap().block_on(async { /* your body */ })
//     }
// `#[tokio::main(flavor = "current_thread")]` uses one thread instead.
#[tokio::main]
async fn main() {
    timeouts().await;
    select_loop().await;
    cancellation().await;
    cancellation_safety().await;
}

async fn slow_answer(ms: u64) -> u32 {
    sleep(Duration::from_millis(ms)).await;
    42
}

async fn timeouts() {
    println!("--- 1. tokio::time::timeout ---");
    // Ok(value) if the future finished in time; Err(Elapsed) if not, and
    // then the future is dropped (cancelled).
    println!(
        "  fast: {:?}",
        timeout(Duration::from_millis(50), slow_answer(5)).await
    );
    println!(
        "  slow: {:?}",
        timeout(Duration::from_millis(10), slow_answer(500)).await
    );
}

async fn select_loop() {
    println!("--- 2. tokio::select!: an event loop ---");
    let (tx, mut rx) = mpsc::channel::<String>(8);
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        for word in ["alpha", "beta", "gamma"] {
            sleep(Duration::from_millis(7)).await;
            tx.send(word.to_string()).await.unwrap();
        }
        sleep(Duration::from_millis(10)).await;
        let _ = shutdown_tx.send(()); // ask the loop to stop
    });

    let mut heartbeat = interval(Duration::from_millis(10));
    loop {
        // Waits on ALL branches at once; the first one ready runs, and the
        // others are dropped (cancelled) for this round. `biased;` checks
        // the branches top to bottom instead of in random order, so the
        // shutdown signal always wins a tie.
        tokio::select! {
            biased;
            _ = &mut shutdown_rx => {
                println!("  shutdown signal: leaving the loop");
                break;
            }
            Some(message) = rx.recv() => println!("  message: {message}"),
            _ = heartbeat.tick() => println!("  (heartbeat)"),
        }
    }
}

/// Prints when it's dropped, to show exactly when cancellation happens.
struct Noisy(&'static str);
impl Drop for Noisy {
    fn drop(&mut self) {
        println!("  dropped: {}", self.0);
    }
}

async fn cancellation() {
    println!("--- 3. cancellation: dropping a future stops it ---");
    let work = async {
        let _guard = Noisy("the slow future (cleanup runs on cancel)");
        sleep(Duration::from_millis(500)).await;
        println!("  never printed: the future is cancelled at its .await");
    };
    let _ = timeout(Duration::from_millis(5), work).await;

    // Tasks are cancelled with `abort()`; `JoinSet` manages a group of them.
    let mut set = JoinSet::new();
    for (name, ms) in [("quick", 5), ("medium", 40), ("forever", 10_000)] {
        set.spawn(async move {
            sleep(Duration::from_millis(ms)).await;
            name
        });
    }
    for _ in 0..2 {
        if let Some(Ok(name)) = set.join_next().await {
            println!("  JoinSet: '{name}' finished");
        }
    }
    set.abort_all(); // stop the rest (dropping the JoinSet does this too)
    while let Some(result) = set.join_next().await {
        println!(
            "  JoinSet: cancelled? {}",
            result.unwrap_err().is_cancelled()
        );
    }
}

/// Collects three messages. NOT cancellation safe: if this future is
/// dropped half-way, the messages already in `batch` are lost.
async fn next_batch(rx: &mut mpsc::Receiver<u32>) -> Vec<u32> {
    let mut batch = Vec::new();
    while batch.len() < 3 {
        match rx.recv().await {
            Some(n) => batch.push(n),
            None => break,
        }
    }
    batch
}

async fn cancellation_safety() {
    println!("--- 4. cancellation safety ---");
    // A future used in a `select!` loop is dropped whenever ANOTHER branch
    // wins. `rx.recv()` is cancellation safe: a message is either returned
    // or still in the channel. `next_batch` is not.
    for fixed in [false, true] {
        let (tx, mut rx) = mpsc::channel(32);
        tokio::spawn(async move {
            for n in 0..12 {
                tx.send(n).await.unwrap();
                sleep(Duration::from_millis(3)).await;
            }
        });
        let mut ticker = interval(Duration::from_millis(4));
        let mut received = Vec::new();

        if fixed {
            // FIX: create the future ONCE, pin it, and poll it by reference
            // (`&mut batch`), so it survives the rounds the ticker wins.
            loop {
                let mut batch = pin!(next_batch(&mut rx));
                let done = loop {
                    tokio::select! {
                        b = &mut batch => break b,
                        _ = ticker.tick() => {} // ticker wins; `batch` lives on
                    }
                };
                if done.is_empty() {
                    break;
                }
                received.extend(done);
            }
        } else {
            // BUG: a NEW next_batch future each round; when the ticker wins,
            // the half-collected batch is dropped with it.
            loop {
                tokio::select! {
                    batch = next_batch(&mut rx) => {
                        if batch.is_empty() { break; }
                        received.extend(batch);
                    }
                    _ = ticker.tick() => {}
                }
            }
        }
        let label = if fixed {
            "pinned once "
        } else {
            "recreated   "
        };
        println!("  {label}: received {} of 12 messages", received.len());
    }
}
