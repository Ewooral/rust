//! Mini project for Book chapter 17: an async price checker.
//!
//! It asks several (simulated) shops for the price of an item, all at the
//! same time, the way a comparison site queries real shop APIs:
//!
//! - every request has a **timeout**, and failed requests are **retried**
//!   with a growing pause (exponential backoff): `quote_with_retry`;
//! - every shop runs in its own **task** (`tokio::spawn` inside a
//!   `JoinSet`), sharing the shop list as `Arc<dyn Shop>`;
//! - a **semaphore** limits how many requests are in flight at once;
//! - an overall **deadline** in `tokio::select!` cancels the stragglers:
//!   `compare_prices`;
//! - `first_quote` **races** the shops and keeps the first good answer,
//!   cancelling the rest.
//!
//! Shops are simulated with `tokio::time::sleep`, so the tests run on
//! Tokio's paused test clock: instant, and never flaky.

use futures::StreamExt;
use futures::future::BoxFuture;
use futures::stream::FuturesUnordered;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::{Instant, sleep, sleep_until, timeout};

/// A price in pence, so no floating-point rounding.
pub type Pence = u32;

/// Why a shop gave no price.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchError {
    /// No answer within the per-request timeout (after every retry).
    TimedOut,
    /// The shop answered with an error (after every retry).
    Unavailable,
    /// The shop answered: it doesn't sell this item. Not retried.
    OutOfStock,
    /// The overall deadline passed first, so the request was cancelled.
    Cancelled,
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            FetchError::TimedOut => "timed out",
            FetchError::Unavailable => "unavailable",
            FetchError::OutOfStock => "out of stock",
            FetchError::Cancelled => "cancelled at the deadline",
        };
        f.write_str(text)
    }
}

/// A shop we can ask for prices. `Send + Sync` so one `Arc<dyn Shop>` can
/// be shared by many tasks; the method returns a boxed future so the trait
/// works as a `dyn` trait object (see the `async_traits_and_shared_state`
/// example).
pub trait Shop: Send + Sync {
    fn name(&self) -> &str;
    fn price<'a>(&'a self, item: &'a str) -> BoxFuture<'a, Result<Pence, FetchError>>;
}

/// A pretend shop: answers after `latency`, can be told to fail its first
/// few requests, and knows a few prices.
pub struct SimulatedShop {
    name: String,
    latency: Duration,
    prices: HashMap<String, Pence>,
    // A std Mutex is fine in async code when the guard is never held
    // across an `.await`.
    failures_left: Mutex<u32>,
}

impl SimulatedShop {
    pub fn new(name: &str, latency_ms: u64, prices: &[(&str, Pence)]) -> Self {
        SimulatedShop {
            name: name.to_string(),
            latency: Duration::from_millis(latency_ms),
            prices: prices
                .iter()
                .map(|(item, p)| (item.to_string(), *p))
                .collect(),
            failures_left: Mutex::new(0),
        }
    }

    /// Makes the shop fail its first `n` requests (a flaky server).
    pub fn failing_first(self, n: u32) -> Self {
        *self.failures_left.lock().unwrap() = n;
        self
    }
}

impl Shop for SimulatedShop {
    fn name(&self) -> &str {
        &self.name
    }

    fn price<'a>(&'a self, item: &'a str) -> BoxFuture<'a, Result<Pence, FetchError>> {
        Box::pin(async move {
            sleep(self.latency).await; // the "network"
            let failed = {
                let mut left = self.failures_left.lock().unwrap();
                if *left > 0 {
                    *left -= 1;
                    true
                } else {
                    false
                }
            }; // the guard is dropped here, before any other `.await`
            if failed {
                return Err(FetchError::Unavailable);
            }
            self.prices.get(item).copied().ok_or(FetchError::OutOfStock)
        })
    }
}

/// How patient to be.
#[derive(Debug, Clone)]
pub struct Config {
    /// The longest one request may take.
    pub request_timeout: Duration,
    /// Extra attempts after the first one fails or times out.
    pub retries: u32,
    /// The pause before the first retry; it doubles each time.
    pub backoff: Duration,
    /// At most this many requests at once (at least 1).
    pub max_in_flight: usize,
    /// After this long, unfinished shops are cancelled.
    pub deadline: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            request_timeout: Duration::from_millis(100),
            retries: 2,
            backoff: Duration::from_millis(10),
            max_in_flight: 4,
            deadline: Duration::from_millis(300),
        }
    }
}

/// One shop's answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quote {
    pub shop: String,
    pub price: Result<Pence, FetchError>,
    pub attempts: u32,
}

/// Asks one shop, with a timeout on every attempt and retries with
/// exponential backoff. `OutOfStock` is a real answer, so it isn't retried.
pub async fn quote_with_retry(shop: &dyn Shop, item: &str, config: &Config) -> Quote {
    let mut pause = config.backoff;
    let mut attempts = 0;
    loop {
        attempts += 1;
        let result = match timeout(config.request_timeout, shop.price(item)).await {
            Ok(answer) => answer,
            Err(_elapsed) => Err(FetchError::TimedOut), // the request was dropped
        };
        let retryable = matches!(result, Err(FetchError::TimedOut | FetchError::Unavailable));
        if !retryable || attempts > config.retries {
            return Quote {
                shop: shop.name().to_string(),
                price: result,
                attempts,
            };
        }
        sleep(pause).await;
        pause *= 2;
    }
}

/// Asks every shop concurrently: one task per shop, at most
/// `config.max_in_flight` requesting at once, and anything still running at
/// the deadline is cancelled and reported as `Cancelled`. The quotes come
/// back in the same order as `shops`.
pub async fn compare_prices(shops: &[Arc<dyn Shop>], item: &str, config: &Config) -> Vec<Quote> {
    let deadline = Instant::now() + config.deadline;
    let permits = Arc::new(Semaphore::new(config.max_in_flight.max(1)));
    let mut tasks = JoinSet::new();
    for (index, shop) in shops.iter().enumerate() {
        // Each task OWNS clones of what it needs: `tokio::spawn` requires
        // `'static`, so it can't borrow `shops`, `item` or `config`.
        let shop = Arc::clone(shop);
        let item = item.to_string();
        let config = config.clone();
        let permits = Arc::clone(&permits);
        tasks.spawn(async move {
            // Wait for a free slot; the permit is released when dropped.
            let _permit = permits.acquire_owned().await.expect("semaphore closed");
            (index, quote_with_retry(&*shop, &item, &config).await)
        });
    }

    let mut answers: Vec<Option<Quote>> = vec![None; shops.len()];
    loop {
        tokio::select! {
            finished = tasks.join_next() => match finished {
                Some(Ok((index, quote))) => answers[index] = Some(quote),
                Some(Err(e)) => panic!("a shop task panicked: {e}"),
                None => break, // every task is done
            },
            _ = sleep_until(deadline) => {
                tasks.abort_all(); // cancel the stragglers
                break;
            }
        }
    }

    answers
        .into_iter()
        .zip(shops)
        .map(|(answer, shop)| {
            answer.unwrap_or_else(|| Quote {
                shop: shop.name().to_string(),
                price: Err(FetchError::Cancelled),
                attempts: 0,
            })
        })
        .collect()
}

/// Races every shop (one attempt each, with the request timeout) and
/// returns the FIRST successful quote. Returning drops the other requests,
/// which cancels them. `None` if no shop could answer.
pub async fn first_quote(shops: &[Arc<dyn Shop>], item: &str, config: &Config) -> Option<Quote> {
    let mut racing: FuturesUnordered<_> = shops
        .iter()
        .map(|shop| async move {
            let price = timeout(config.request_timeout, shop.price(item))
                .await
                .unwrap_or(Err(FetchError::TimedOut));
            Quote {
                shop: shop.name().to_string(),
                price,
                attempts: 1,
            }
        })
        .collect();
    while let Some(quote) = racing.next().await {
        if quote.price.is_ok() {
            return Some(quote);
        }
    }
    None
}

/// The cheapest successful quote, if any.
pub fn cheapest(quotes: &[Quote]) -> Option<&Quote> {
    quotes
        .iter()
        .filter(|q| q.price.is_ok())
        .min_by_key(|q| *q.price.as_ref().unwrap_or(&Pence::MAX))
}

/// Formats pence as pounds: 1299 -> "£12.99".
pub fn pounds(pence: Pence) -> String {
    format!("£{}.{:02}", pence / 100, pence % 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shop(name: &str, latency_ms: u64, price: Pence) -> SimulatedShop {
        SimulatedShop::new(name, latency_ms, &[("kettle", price)])
    }

    fn shared(shops: Vec<SimulatedShop>) -> Vec<Arc<dyn Shop>> {
        shops
            .into_iter()
            .map(|s| Arc::new(s) as Arc<dyn Shop>)
            .collect()
    }

    // `start_paused = true`: Tokio's clock is frozen and jumps forward
    // whenever every task is waiting on a timer, so `sleep(100 ms)` takes no
    // real time and the elapsed times below are exact, never flaky.

    #[tokio::test(start_paused = true)]
    async fn a_flaky_shop_succeeds_after_retries() {
        let flaky = shop("Flaky", 10, 500).failing_first(2);
        let quote = quote_with_retry(&flaky, "kettle", &Config::default()).await;
        assert_eq!(quote.price, Ok(500));
        assert_eq!(quote.attempts, 3);
    }

    #[tokio::test(start_paused = true)]
    async fn retries_run_out() {
        let flaky = shop("Flaky", 10, 500).failing_first(5);
        let config = Config {
            retries: 1,
            ..Config::default()
        };
        let quote = quote_with_retry(&flaky, "kettle", &config).await;
        assert_eq!(quote.price, Err(FetchError::Unavailable));
        assert_eq!(quote.attempts, 2);
    }

    #[tokio::test(start_paused = true)]
    async fn a_slow_shop_times_out_and_out_of_stock_is_not_retried() {
        let config = Config::default(); // 100 ms timeout, 2 retries
        let start = Instant::now();
        let slow = quote_with_retry(&shop("Slow", 1_000, 1), "kettle", &config).await;
        assert_eq!(slow.price, Err(FetchError::TimedOut));
        assert_eq!(slow.attempts, 3);
        // 3 timeouts of 100 ms plus backoffs of 10 + 20 ms = 330 ms
        let took = start.elapsed();
        assert!(
            took >= Duration::from_millis(330) && took < Duration::from_millis(340),
            "{took:?}"
        );

        let none = quote_with_retry(&shop("Shop", 5, 1), "teapot", &config).await;
        assert_eq!(none.price, Err(FetchError::OutOfStock));
        assert_eq!(none.attempts, 1);
    }

    #[tokio::test(start_paused = true)]
    async fn shops_are_asked_concurrently_in_input_order() {
        let shops = shared(vec![
            shop("A", 80, 300),
            shop("B", 20, 200),
            shop("C", 50, 100),
        ]);
        let start = Instant::now();
        let quotes = compare_prices(&shops, "kettle", &Config::default()).await;
        // Concurrent: as long as the slowest shop (80 ms), not the sum (150 ms).
        assert!(
            start.elapsed() < Duration::from_millis(100),
            "{:?}",
            start.elapsed()
        );
        let names: Vec<&str> = quotes.iter().map(|q| q.shop.as_str()).collect();
        assert_eq!(names, ["A", "B", "C"]);
        assert_eq!(cheapest(&quotes).unwrap().shop, "C");
    }

    #[tokio::test(start_paused = true)]
    async fn the_semaphore_limits_requests_in_flight() {
        let shops = shared(vec![shop("A", 50, 1), shop("B", 50, 2), shop("C", 50, 3)]);
        let config = Config {
            max_in_flight: 1,
            ..Config::default()
        };
        let start = Instant::now();
        let quotes = compare_prices(&shops, "kettle", &config).await;
        assert!(quotes.iter().all(|q| q.price.is_ok()));
        assert!(
            start.elapsed() >= Duration::from_millis(150),
            "one at a time"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn the_deadline_cancels_stragglers() {
        let shops = shared(vec![shop("Quick", 10, 100), shop("Slow", 90, 50)]);
        let config = Config {
            request_timeout: Duration::from_millis(500),
            deadline: Duration::from_millis(50),
            ..Config::default()
        };
        let start = Instant::now();
        let quotes = compare_prices(&shops, "kettle", &config).await;
        let took = start.elapsed();
        assert!(
            took >= Duration::from_millis(50) && took < Duration::from_millis(60),
            "{took:?}"
        );
        assert_eq!(quotes[0].price, Ok(100));
        assert_eq!(quotes[1].price, Err(FetchError::Cancelled));
    }

    #[tokio::test(start_paused = true)]
    async fn first_quote_skips_failures_and_takes_the_fastest_success() {
        let shops = shared(vec![
            shop("Steady", 60, 900),
            shop("Broken", 5, 1).failing_first(10),
            shop("Speedy", 30, 999),
        ]);
        let quote = first_quote(&shops, "kettle", &Config::default())
            .await
            .unwrap();
        assert_eq!(quote.shop, "Speedy");
        assert!(
            first_quote(&shops, "teapot", &Config::default())
                .await
                .is_none()
        );
    }

    #[test]
    fn formatting_pence() {
        assert_eq!(pounds(1299), "£12.99");
        assert_eq!(pounds(5), "£0.05");
        assert_eq!(FetchError::TimedOut.to_string(), "timed out");
    }
}
