//! Book 17.1: futures and the async syntax.
//!
//! - An `async fn` returns a FUTURE: a value that will be ready later.
//! - Futures are LAZY: nothing happens until something `.await`s them.
//! - `.await` only works inside async code, and a RUNTIME drives it all.
//!
//! The Book downloads real web pages with `trpl::get`. To stay offline and
//! fast, this example SIMULATES slow pages with `tokio::time::sleep`.
//!
//! Run: cargo run -p async_await --example futures_and_syntax

use futures::future::{self, Either};
use std::pin::pin;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Pretend to download a page: wait `ms` milliseconds, then "parse" the
/// title out of the last part of the URL.
async fn page_title(url: &str, ms: u64) -> Option<String> {
    sleep(Duration::from_millis(ms)).await;
    let last = url.rsplit('/').next()?;
    (!last.is_empty()).then(|| last.replace('-', " "))
}

/// Roughly what the compiler turns `async fn page_title(..) -> Option<String>`
/// into: an ordinary fn that returns `impl Future<Output = Option<String>>`,
/// with the body moved into an `async` block. (Clippy would suggest
/// `async fn` here: writing it out by hand is the point.)
#[allow(clippy::manual_async_fn)]
fn page_title_desugared(url: &str, ms: u64) -> impl Future<Output = Option<String>> + '_ {
    async move {
        sleep(Duration::from_millis(ms)).await;
        let last = url.rsplit('/').next()?;
        (!last.is_empty()).then(|| last.replace('-', " "))
    }
}

async fn announce(message: &str) {
    println!("   (announce runs now: {message})");
}

fn main() {
    // 1. Calling an async fn runs NONE of its body. It only builds a future.
    // (If you never use it, the compiler warns: "futures do nothing unless
    // you `.await` or poll them".)
    let lazy = announce("I only run when awaited");
    println!("1. built a future; nothing has been announced yet");

    // 2. Something has to DRIVE futures: a runtime (also called an executor).
    // `main` itself can't be async without help:
    //     async fn main() {} // E0752: `main` function is not allowed to be `async`
    // so we start a Tokio runtime and hand it our top-level future. This is
    // exactly what the Book's `trpl::block_on` (formerly `trpl::run`) does.
    let runtime = tokio::runtime::Runtime::new().expect("could not start Tokio");
    runtime.block_on(async {
        println!("2. inside the runtime; now we await the lazy future:");
        lazy.await;

        // 3. `.await` pauses THIS async block until the page is ready.
        // Outside async code it's an error:
        //     fn not_async() { page_title("u", 1).await; } // E0728: `await` is only allowed inside `async` functions and blocks
        let start = Instant::now();
        let title = page_title("https://example.com/rust-is-fun", 30).await;
        println!("3. got {title:?} after {:?}", start.elapsed());

        // `.await` is POSTFIX, so chains read left to right.
        let length = page_title_desugared("https://example.com/the-desugared-version", 10)
            .await
            .map(|t| t.len());
        println!("   desugared version: title length {length:?}");

        // 4. The Book's race: start two downloads and take whichever finishes
        // first. `select` returns `Either::Left` or `Either::Right`. (`pin!`
        // is explained in 17.5; `select` needs its futures pinned.)
        let slow = pin!(page_title("https://slow.example/the-slow-site", 60));
        let fast = pin!(page_title("https://fast.example/the-fast-site", 15));
        let start = Instant::now();
        let winner = match future::select(slow, fast).await {
            Either::Left((title, _still_running)) => ("slow", title),
            Either::Right((title, _still_running)) => ("fast", title),
        };
        println!(
            "4. the {} site won with {:?} after {:?}",
            winner.0,
            winner.1,
            start.elapsed()
        );

        // 5. An `async` BLOCK is a future too, written inline.
        let block = async {
            let a = page_title("https://a.example/first", 5).await;
            let b = page_title("https://b.example/second", 5).await;
            format!("{a:?} then {b:?}")
        };
        println!("5. async block result: {}", block.await);
    });

    // 6. `#[tokio::main]` on `async fn main` writes the runtime setup above
    // for you. Every other example in this lesson uses it.
    println!("6. done: the runtime has been shut down");
}
