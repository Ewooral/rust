//! Spans in async code: `#[instrument]` and `.instrument(span)`, and why
//! `span.enter()` must never be held across an `.await`.
//!
//! Run: cargo run -p observability --example async_spans

use std::time::Duration;

use tokio::time::sleep;
use tracing::{Instrument, info, info_span, instrument};

/// `#[instrument]` on an async fn instruments the FUTURE: the span is
/// entered each time the future is polled and exited each time it pauses.
#[instrument]
async fn fetch(url: &'static str, ms: u64) -> usize {
    info!("start");
    sleep(Duration::from_millis(ms)).await;
    info!("done");
    url.len()
}

/// WRONG: the guard keeps the span entered on this THREAD while the future
/// is paused, so whatever else runs on the thread meanwhile is logged
/// inside this span.
async fn wrong(name: &'static str) {
    let span = info_span!("wrong", name);
    let _guard = span.enter();
    sleep(Duration::from_millis(20)).await;
    info!("finished");
}

/// Logs one line after 10 ms. It belongs to no span at all.
async fn innocent() {
    sleep(Duration::from_millis(10)).await;
    info!("I belong to no span");
}

/// RIGHT: attach the span to the future instead.
async fn right(name: &'static str) {
    async {
        sleep(Duration::from_millis(20)).await;
        info!("finished");
    }
    .instrument(info_span!("right", name))
    .await;
}

// One thread, so the mistake shows up the same way every run.
#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .init();

    println!("--- two #[instrument]ed fetches, interleaved");
    tokio::join!(
        fetch("https://a.example", 20),
        fetch("https://bb.example", 10)
    );

    println!("--- span.enter() held across .await (wrong)");
    tokio::join!(wrong("a"), innocent());

    println!("--- .instrument(span) (right)");
    tokio::join!(right("a"), innocent());

    println!("--- tokio::spawn: a task starts with NO span unless you give it one");
    let parent = info_span!("batch");
    async {
        let lost = tokio::spawn(async { info!("spawned plainly") });
        let kept = tokio::spawn(async { info!("spawned with in_current_span") }.in_current_span());
        let _ = tokio::join!(lost, kept);
    }
    .instrument(parent)
    .await;
}
