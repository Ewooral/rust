//! Exercise 2: make it compile (E0308).
//!
//! Three futures go into one `Vec` for `join_all`. But every `async` block
//! has its own unique, unnamed type, even when two look identical, so they
//! can't share a `Vec`. Give the Vec ONE element type: box and pin each
//! future, and annotate the Vec as a list of trait objects:
//! `Vec<Pin<Box<dyn Future<Output = ()>>>>`. (Add `use std::pin::Pin;`.)
//!
//! Run: cargo run -p async_await --features exercises --bin ex02_fix_mismatched_futures

use futures::future::join_all;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let tx_fast = tx.clone();

    let fast = async move {
        for n in 1..=3 {
            tx_fast.send(format!("fast {n}")).unwrap();
            sleep(Duration::from_millis(10)).await;
        }
    };
    let slow = async move {
        for n in 1..=2 {
            tx.send(format!("slow {n}")).unwrap();
            sleep(Duration::from_millis(25)).await;
        }
    };
    let printer = async {
        while let Some(message) = rx.recv().await {
            println!("received {message}");
        }
    };

    let futures = vec![fast, slow, printer];
    join_all(futures).await;
}
