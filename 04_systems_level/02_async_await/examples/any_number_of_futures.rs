//! Book 17.3: working with any number of futures: `join_all`, racing with
//! `select`, yielding control, and building our own `timeout`.
//!
//! Run: cargo run -p async_await --example any_number_of_futures

use futures::future::{self, Either, join_all};
use std::pin::{Pin, pin};
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    many_futures().await;
    racing().await;
    yielding().await;
    our_own_timeout().await;
}

async fn many_futures() {
    println!("--- 1. any number of futures ---");
    // `join!` needs to know HOW MANY futures when you write the code, but it
    // can mix output types: here a number, some text and a bool.
    let (n, s, b) = futures::join!(async { 1 }, async { "two" }, async { true });
    println!("  join! of three different types: {n}, {s}, {b}");

    // `join_all` takes a whole collection. Every async block has its OWN
    // anonymous type, so putting different ones in one Vec fails:
    //     let futures = vec![async { 1 }, async { 2 }];
    //     // E0308: mismatched types: expected `async` block, found a different `async` block
    // The fix: put each one in a box and use a trait object (with `Pin`,
    // explained in 17.5).
    let futures: Vec<Pin<Box<dyn Future<Output = u64>>>> = vec![
        Box::pin(async {
            sleep(Duration::from_millis(30)).await;
            1
        }),
        Box::pin(async { 2 }),
        Box::pin(async {
            sleep(Duration::from_millis(10)).await;
            3
        }),
    ];
    let start = Instant::now();
    let results = join_all(futures).await; // results keep the input order
    println!(
        "  join_all: {results:?} in {:?} (not 40 ms: they overlap)",
        start.elapsed()
    );

    // Same type? Then no boxing needed: every call of one async fn returns
    // the same type.
    let delays = [25, 5, 15];
    let doubled = join_all(delays.iter().map(|&ms| double_later(ms))).await;
    println!("  join_all over a map: {doubled:?}");
}

async fn double_later(ms: u64) -> u64 {
    sleep(Duration::from_millis(ms)).await;
    ms * 2
}

async fn racing() {
    println!("--- 2. racing: the first to finish wins ---");
    let slow = pin!(async {
        sleep(Duration::from_millis(40)).await;
        "slow"
    });
    let fast = pin!(async {
        sleep(Duration::from_millis(10)).await;
        "fast"
    });
    match future::select(slow, fast).await {
        Either::Left((winner, _)) | Either::Right((winner, _)) => println!("  winner: {winner}"),
    }
    // The loser is DROPPED here, mid-sleep. Dropping a future cancels it.

    // `select_all` races any number (they must be Unpin, so box them).
    let racers = [30u64, 10, 20].map(|ms| Box::pin(double_later(ms)));
    let (value, index, _rest) = future::select_all(racers).await;
    println!("  select_all: racer #{index} finished first with {value}");
}

/// A future that does slow work WITHOUT awaiting anything (we pretend with
/// `std::thread::sleep`, which blocks the whole thread).
fn slow(name: &str, ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
    println!("  '{name}' ran for {ms} ms");
}

async fn yielding() {
    println!("--- 3. yielding: let the others have a go ---");
    // Futures only switch at `.await` points. Without one, `a` hogs the
    // thread until it's completely done, and `b` can't even start.
    let a = async {
        println!("  'a' started");
        slow("a", 5);
        slow("a", 5);
        "a"
    };
    let b = async {
        println!("  'b' started");
        slow("b", 5);
        "b"
    };
    let winner = match future::select(pin!(a), pin!(b)).await {
        Either::Left((w, _)) | Either::Right((w, _)) => w,
    };
    println!("  without yields: '{winner}' finished before 'b' did anything else");

    // `yield_now().await` (the Book's `trpl::yield_now`) hands control back
    // to the runtime, so other futures make progress in between.
    let a = async {
        println!("  'a' started");
        slow("a", 5);
        tokio::task::yield_now().await;
        slow("a", 5);
        tokio::task::yield_now().await;
        "a"
    };
    let b = async {
        println!("  'b' started");
        slow("b", 5);
        tokio::task::yield_now().await;
        "b"
    };
    let winner = match future::select(pin!(a), pin!(b)).await {
        Either::Left((w, _)) | Either::Right((w, _)) => w,
    };
    println!("  with yields: they took turns, and '{winner}' won");
}

/// The Book builds this from smaller pieces: race the future against a
/// sleep. `Ok(value)` if it finished in time, `Err(max)` if not.
async fn timeout<F: Future>(future_to_try: F, max: Duration) -> Result<F::Output, Duration> {
    match future::select(pin!(future_to_try), pin!(sleep(max))).await {
        Either::Left((output, _)) => Ok(output),
        Either::Right(((), _)) => Err(max),
    }
}

async fn our_own_timeout() {
    println!("--- 4. our own timeout, built from select + sleep ---");
    let quick = timeout(double_later(5), Duration::from_millis(50)).await;
    let too_slow = timeout(double_later(200), Duration::from_millis(20)).await;
    println!("  quick:    {quick:?}");
    println!("  too slow: {too_slow:?}");
    // Tokio ships a ready-made one: tokio::time::timeout (see tokio_in_practice).
}
