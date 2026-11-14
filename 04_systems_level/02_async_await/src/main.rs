//! Mini project for Book chapter 17: an async price checker. It asks six
//! simulated shops for the price of a kettle, all at once, with timeouts,
//! retries, a limit on requests in flight and an overall deadline. The
//! logic lives in `src/lib.rs`.
//!
//!   cargo run -p async_await                 look up "kettle"
//!   cargo run -p async_await -- toaster      look up something else
//!   cargo test -p async_await

use async_await::{Config, Shop, SimulatedShop, cheapest, compare_prices, first_quote, pounds};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;

fn shops() -> Vec<Arc<dyn Shop>> {
    let shops = vec![
        SimulatedShop::new("Corner Shop", 40, &[("kettle", 2499), ("toaster", 1999)]),
        SimulatedShop::new("MegaMart", 80, &[("kettle", 1899), ("toaster", 2250)]),
        // Fails its first request, then answers: saved by a retry.
        SimulatedShop::new("Flaky Bazaar", 30, &[("kettle", 1750)]).failing_first(1),
        // Slower than the request timeout every time, so it keeps retrying
        // until the overall deadline cancels it.
        SimulatedShop::new("Snail Supplies", 150, &[("kettle", 999)]),
        // Fast, but sells no kettles.
        SimulatedShop::new("Toast Palace", 20, &[("toaster", 1500)]),
        // Always broken.
        SimulatedShop::new("Closed Co", 10, &[("kettle", 500)]).failing_first(u32::MAX),
    ];
    shops
        .into_iter()
        .map(|s| Arc::new(s) as Arc<dyn Shop>)
        .collect()
}

#[tokio::main]
async fn main() {
    let item = env::args().nth(1).unwrap_or_else(|| "kettle".to_string());
    let shops = shops();
    let config = Config {
        request_timeout: Duration::from_millis(100),
        retries: 2,
        backoff: Duration::from_millis(10),
        max_in_flight: 4,
        deadline: Duration::from_millis(250),
    };
    println!(
        "Asking {} shops for a {item}: {:?} per request, {} retries, {} at once, {:?} deadline\n",
        shops.len(),
        config.request_timeout,
        config.retries,
        config.max_in_flight,
        config.deadline
    );

    let start = Instant::now();
    let quotes = compare_prices(&shops, &item, &config).await;
    let took = start.elapsed();

    println!("| shop           | price                     | attempts |");
    println!("|----------------|---------------------------|----------|");
    for quote in &quotes {
        let price = match &quote.price {
            Ok(pence) => pounds(*pence),
            Err(e) => e.to_string(),
        };
        println!(
            "| {:<14} | {:<25} | {:>8} |",
            quote.shop, price, quote.attempts
        );
    }
    match cheapest(&quotes) {
        Some(best) => println!(
            "\nCheapest: {} at {}",
            best.shop,
            pounds(*best.price.as_ref().expect("cheapest only returns prices"))
        ),
        None => println!("\nNobody sells a {item}."),
    }
    println!("All of that took {took:?}. Every shop was asked at the same time, so that's");
    println!("the deadline (or the slowest shop), not the sum of all their waits.");

    let start = Instant::now();
    match first_quote(&shops, &item, &config).await {
        Some(quote) => println!(
            "\nRace: {} answered first with {} after {:?} (the others were cancelled).",
            quote.shop,
            pounds(
                *quote
                    .price
                    .as_ref()
                    .expect("first_quote only returns prices")
            ),
            start.elapsed()
        ),
        None => println!("\nRace: no shop had a {item}."),
    }
}
