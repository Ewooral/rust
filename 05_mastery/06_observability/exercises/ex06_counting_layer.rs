//! Exercise 6: write a metrics `Layer`.
//!
//! A layer sees every event and span that passes through the subscriber.
//! `CountingLayer` should keep three numbers in its shared `Counts`:
//! - `errors`: events at level ERROR;
//! - `warnings`: events at level WARN;
//! - `requests`: spans named "request" that have CLOSED (finished).
//!
//! Fill in `on_event` and `on_close` (they do nothing yet, so the tests
//! fail with counts of 0). Don't use `todo!()` here: a panic inside a
//! layer, while another panic is unwinding, aborts the whole test run.
//! Hints: `event.metadata().level()` is a `&Level`, compare it with
//! `Level::ERROR`; in `on_close`, `ctx.span(&id)` looks the span up and
//! `.name()` gives its name. Lock the counts with `self.counts.lock()`.
//!
//! Test: cargo test -p observability --features exercises --bin ex06_counting_layer

#![allow(unused_variables, unused_imports, dead_code)]

use std::sync::{Arc, Mutex};

use tracing::{Event, Level, Subscriber, span};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Counts {
    errors: u64,
    warnings: u64,
    requests: u64,
}

struct CountingLayer {
    counts: Arc<Mutex<Counts>>,
}

impl<S> Layer<S> for CountingLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // Count ERROR and WARN events here.
    }

    fn on_close(&self, id: span::Id, ctx: Context<'_, S>) {
        // Count closed spans named "request" here.
    }
}

/// Runs `f` with a CountingLayer installed and returns the counts.
fn count(f: impl FnOnce()) -> Counts {
    let counts = Arc::new(Mutex::new(Counts::default()));
    let subscriber = tracing_subscriber::registry().with(CountingLayer {
        counts: Arc::clone(&counts),
    });
    tracing::subscriber::with_default(subscriber, f);
    *counts.lock().unwrap()
}

fn serve(requests: u32) {
    for id in 0..requests {
        let _span = tracing::info_span!("request", id).entered();
        if id % 3 == 0 {
            tracing::error!("internal error");
        } else {
            tracing::info!("ok");
        }
    }
    tracing::warn!("shutting down");
}

fn main() {
    println!("{:?}", count(|| serve(5)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_errors_and_warnings() {
        let counts = count(|| serve(7));
        assert_eq!((counts.errors, counts.warnings), (3, 1));
    }

    #[test]
    fn counts_only_closed_request_spans() {
        let counts = count(|| {
            serve(4);
            let _other = tracing::info_span!("not_a_request").entered();
        });
        assert_eq!(counts.requests, 4);
    }

    #[test]
    fn info_events_are_not_counted() {
        assert_eq!(count(|| tracing::info!("hello")), Counts::default());
    }
}
