//! Solution to exercise 1: make it compile (E0382: use of moved value: `tx`), then
//! make it finish.
//!
//! Three producer tasks should each send two messages into ONE mpsc
//! channel. But `async move` moves `tx` into the first task, so the second
//! time round the loop there is no `tx` left.
//!
//! 1. Give every task its own sender: clone `tx` inside the loop, before
//!    the `tokio::spawn`. (Cloning a Sender is cheap: it's a handle to the
//!    same channel.)
//! 2. Run it. It now compiles, but prints "timed out": `rx.recv()` only
//!    returns `None` when EVERY sender is gone, and `main` still holds the
//!    original `tx`. Drop it after the loop.
//!
//! Run: cargo run -p tokio_runtime --features solutions --bin sol01_fix_moved_sender

use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<String>(16);

    for id in 1..=3 {
        let tx = tx.clone(); // each task gets its own handle to the channel
        tokio::spawn(async move {
            for n in 1..=2 {
                tx.send(format!("producer {id}: message {n}"))
                    .await
                    .unwrap();
            }
        });
    }
    drop(tx); // main's sender: once the tasks' clones are gone too, recv() gives None

    let receiving = async {
        let mut got = Vec::new();
        while let Some(message) = rx.recv().await {
            got.push(message);
        }
        got
    };
    match timeout(Duration::from_secs(2), receiving).await {
        Ok(mut got) => {
            got.sort(); // tasks run concurrently: sort for a stable order
            println!("received {} messages:", got.len());
            for message in got {
                println!("  {message}");
            }
        }
        Err(_) => println!("timed out: the channel never closed (is a sender still alive?)"),
    }
}
