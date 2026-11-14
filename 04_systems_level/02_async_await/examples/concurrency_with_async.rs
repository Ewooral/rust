//! Book 17.2: applying concurrency with async: tasks, `join`, and async
//! message passing. Compare each part with lesson 04/01: it's the same
//! ideas with futures instead of threads.
//!
//! Run: cargo run -p async_await --example concurrency_with_async

use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

const TICK: Duration = Duration::from_millis(5);

#[tokio::main]
async fn main() {
    // 1. `tokio::spawn` (the Book's `trpl::spawn_task`) is `thread::spawn`
    // for futures: it starts a TASK and returns a JoinHandle you can await.
    println!("--- 1. a spawned task, then await its handle ---");
    let handle = tokio::spawn(async {
        for i in 1..5 {
            println!("  task: hi number {i}");
            sleep(TICK).await;
        }
    });
    for i in 1..3 {
        println!("  main: hi number {i}");
        sleep(TICK).await;
    }
    handle.await.unwrap(); // like join(): wait for the task to finish

    // Like `thread::spawn`, a spawned task may outlive the current function,
    // so it can't borrow locals:
    //     let names = vec![String::from("Ama")];
    //     tokio::spawn(async { println!("{names:?}") });
    //     // E0373: async block may outlive the current function, but it borrows `names`
    // `async move` fixes it, exactly like `move ||` for threads.

    // 2. No task at all: two futures run concurrently on THIS task with
    // `join!`. It polls them in turn, so the output alternates fairly.
    println!("--- 2. join! two futures ---");
    let fut1 = async {
        for i in 1..4 {
            println!("  first:  {i}");
            sleep(TICK).await;
        }
    };
    let fut2 = async {
        for i in 1..4 {
            println!("  second: {i}");
            sleep(TICK).await;
        }
    };
    futures::join!(fut1, fut2); // `tokio::join!` works the same way

    // 3. Message passing. `unbounded_channel` (the Book's `trpl::channel`):
    // `send` is NOT async, `recv` is (it waits for a message).
    println!("--- 3. an async channel ---");
    let (tx, mut rx) = mpsc::unbounded_channel();
    // `async move` takes ownership of `tx`, so when this future finishes, `tx`
    // is dropped, the channel closes, and the `while let` below ends. Without
    // `move`, `tx` lives until the end of main and the loop waits forever.
    let sender = async move {
        for word in ["hi", "from", "the", "future"] {
            tx.send(word.to_string()).unwrap();
            sleep(TICK).await;
        }
    };
    let receiver = async {
        while let Some(word) = rx.recv().await {
            println!("  received '{word}'");
        }
    };
    // If you just awaited `sender` and then `receiver`, every message would
    // be sent before any was received. Joining them interleaves the two.
    futures::join!(sender, receiver);

    // 4. Several producers: clone the sender, one per future.
    println!("--- 4. two producers, one receiver ---");
    let (tx, mut rx) = mpsc::unbounded_channel();
    let tx1 = tx.clone();
    let fast = async move {
        for n in 1..=3 {
            tx1.send(format!("fast {n}")).unwrap();
            sleep(TICK).await;
        }
    };
    let slow = async move {
        for n in 1..=2 {
            tx.send(format!("slow {n}")).unwrap();
            sleep(TICK * 3).await;
        }
    };
    let receiver = async {
        let mut all = Vec::new();
        while let Some(msg) = rx.recv().await {
            all.push(msg);
        }
        all
    };
    let ((), (), all) = futures::join!(fast, slow, receiver);
    println!("  received in arrival order: {all:?}");
}
