//! Solution to exercise 4: run many jobs concurrently, but at most `limit` at once.
//!
//! `fetch` pretends to download a URL (it sleeps for 100 ms). Implement
//! `fetch_all` so that it:
//!
//! - starts every download as its own task (`tokio::spawn` or a `JoinSet`),
//! - never has more than `limit` downloads running at the same time
//!   (a `tokio::sync::Semaphore` with `limit` permits; acquire a permit
//!   INSIDE the task with `acquire_owned()`, and keep it until the fetch is
//!   done),
//! - returns the results in the SAME order as `urls`, even though tasks
//!   finish in any order (remember each task's index).
//!
//! `fetch` records how many downloads run at once in `Stats`; the tests
//! check it. They run on Tokio's paused clock, so 100 ms sleeps take no
//! real time, yet `elapsed()` still shows whether you ran things in
//! parallel: 6 downloads with a limit of 3 should take exactly 200 ms.
//!
//! Test: cargo test -p tokio_runtime --features solutions --bin sol04_limited_fetch

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::sleep;

/// How many fetches are running right now, and the most there ever were.
#[derive(Default)]
struct Stats {
    running: AtomicUsize,
    peak: AtomicUsize,
}

async fn fetch(url: String, stats: Arc<Stats>) -> String {
    let now = stats.running.fetch_add(1, Ordering::SeqCst) + 1;
    stats.peak.fetch_max(now, Ordering::SeqCst);
    sleep(Duration::from_millis(100)).await;
    stats.running.fetch_sub(1, Ordering::SeqCst);
    format!("<html of {url}>")
}

async fn fetch_all(urls: Vec<String>, limit: usize, stats: Arc<Stats>) -> Vec<String> {
    let permits = Arc::new(Semaphore::new(limit));
    let mut set = JoinSet::new();
    for (index, url) in urls.into_iter().enumerate() {
        let permits = Arc::clone(&permits);
        let stats = Arc::clone(&stats);
        set.spawn(async move {
            // An OWNED permit can live inside the task; it is released when
            // `_permit` is dropped at the end of the task.
            let _permit = permits.acquire_owned().await.expect("never closed");
            (index, fetch(url, stats).await)
        });
    }
    // Tasks finish in any order: put each result back in its own slot.
    let mut pages = vec![String::new(); set.len()];
    while let Some(result) = set.join_next().await {
        let (index, page) = result.expect("a fetch task panicked");
        pages[index] = page;
    }
    pages
}

#[tokio::main]
async fn main() {
    let urls: Vec<String> = (1..=6)
        .map(|n| format!("https://example.com/{n}"))
        .collect();
    let stats = Arc::new(Stats::default());
    let start = std::time::Instant::now();
    let pages = fetch_all(urls, 3, Arc::clone(&stats)).await;
    for page in &pages {
        println!("{page}");
    }
    println!(
        "{} pages in about {} ms, at most {} at once",
        pages.len(),
        start.elapsed().as_millis() / 50 * 50,
        stats.peak.load(Ordering::SeqCst)
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    fn urls(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("u{i}")).collect()
    }

    #[tokio::test(start_paused = true)]
    async fn keeps_the_input_order() {
        let pages = fetch_all(urls(5), 2, Arc::default()).await;
        let expected: Vec<String> = urls(5).iter().map(|u| format!("<html of {u}>")).collect();
        assert_eq!(pages, expected);
    }

    #[tokio::test(start_paused = true)]
    async fn never_more_than_the_limit_at_once() {
        let stats = Arc::new(Stats::default());
        fetch_all(urls(10), 3, Arc::clone(&stats)).await;
        assert_eq!(stats.peak.load(Ordering::SeqCst), 3);
        assert_eq!(stats.running.load(Ordering::SeqCst), 0);
    }

    #[tokio::test(start_paused = true)]
    async fn runs_concurrently() {
        let start = Instant::now();
        let pages = fetch_all(urls(6), 3, Arc::default()).await;
        assert_eq!(pages.len(), 6);
        assert_eq!(
            start.elapsed(),
            Duration::from_millis(200),
            "two rounds of three"
        );

        let start = Instant::now();
        fetch_all(urls(6), 6, Arc::default()).await;
        assert_eq!(start.elapsed(), Duration::from_millis(100), "all at once");

        assert!(fetch_all(Vec::new(), 3, Arc::default()).await.is_empty());
    }
}
