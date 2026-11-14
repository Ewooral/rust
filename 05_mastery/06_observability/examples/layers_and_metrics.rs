//! Subscribers are built from layers; each layer sees every span and event.
//!
//! Run: cargo run -p observability --example layers_and_metrics

use observability::metrics::Metrics;
use tracing::level_filters::LevelFilter;
use tracing::{Event, Subscriber, debug, info, info_span, warn};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;

/// The smallest useful layer: prints the name of every span that closes.
struct SpanCloseLogger;

impl<S> Layer<S> for SpanCloseLogger
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_close(&self, id: tracing::span::Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(&id) {
            println!("    [SpanCloseLogger] span `{}` closed", span.name());
        }
    }

    fn on_event(&self, _event: &Event<'_>, _ctx: Context<'_, S>) {}
}

fn handle_requests() {
    for id in 1..=3 {
        let _job = info_span!("job", id).entered();
        debug!("parsing request");
        if id == 2 {
            warn!(outcome = "failed", "bad request");
        } else {
            info!(outcome = "succeeded", "ok");
        }
    }
}

fn main() {
    let metrics = Metrics::default();
    let subscriber = tracing_subscriber::registry()
        // Layer 1: text output, with its OWN filter: INFO and above.
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_target(false)
                .with_filter(LevelFilter::INFO),
        )
        // Layer 2: metrics, unfiltered: it counts the DEBUG events too.
        .with(metrics.layer())
        // Layer 3: our own tiny layer.
        .with(SpanCloseLogger);

    tracing::subscriber::with_default(subscriber, handle_requests);

    let m = metrics.snapshot();
    println!("metrics: events {:?}", m.events);
    println!("         outcomes {:?}", m.outcomes);
    println!("         {} `job` spans closed", m.jobs_finished);
}
