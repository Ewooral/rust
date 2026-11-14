//! Exercise 1: make it compile (E0728).
//!
//! `make_tea` uses `.await`, but it's an ordinary function, and `.await`
//! only works inside `async` functions and blocks. Make `make_tea` async.
//! The compiler then points at one more place: an async fn returns a
//! FUTURE, and futures are lazy, so its caller has to `.await` it too.
//! Leave `main` as a normal fn with `block_on`.
//!
//! Run: cargo run -p async_await --features exercises --bin ex01_fix_await_outside_async

use std::time::Duration;
use tokio::time::sleep;

async fn boil_water() -> &'static str {
    sleep(Duration::from_millis(30)).await;
    "freshly boiled water"
}

fn make_tea() -> String {
    let water = boil_water().await;
    format!("a cup of tea, made with {water}")
}

fn main() {
    let runtime = tokio::runtime::Runtime::new().expect("could not start Tokio");
    let tea = runtime.block_on(async { make_tea() });
    println!("{tea}");
}
