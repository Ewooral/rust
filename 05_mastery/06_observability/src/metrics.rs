//! Metrics from traces: a `Layer` that counts what flows past it.
//!
//! A subscriber is built from layers, and every layer sees every span and
//! event. The `fmt` layer turns them into text; this one turns them into
//! numbers instead:
//! - events per level (how many warnings and errors today?);
//! - job outcomes, read from the structured `outcome` field;
//! - how many `job` spans closed, and the time they were open in total.
//!
//! Real services export numbers like these to Prometheus or OpenTelemetry
//! (see the README); this layer keeps them in memory so we can print and
//! test them.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber, span};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

/// The numbers collected so far.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MetricsSnapshot {
    /// Events seen, by level name ("INFO", "WARN", ...).
    pub events: BTreeMap<String, u64>,
    /// Events with an `outcome` field, by its value.
    pub outcomes: BTreeMap<String, u64>,
    /// How many `job` spans have closed.
    pub jobs_finished: u64,
    /// The total time those spans were open.
    pub job_time: Duration,
}

/// A handle to shared metrics: give `layer()` to the subscriber, keep the
/// handle to read the numbers.
#[derive(Debug, Clone, Default)]
pub struct Metrics(Arc<Mutex<MetricsSnapshot>>);

impl Metrics {
    /// The layer that fills these metrics in.
    pub fn layer(&self) -> MetricsLayer {
        MetricsLayer {
            metrics: self.clone(),
        }
    }

    /// A copy of the numbers right now.
    pub fn snapshot(&self) -> MetricsSnapshot {
        self.0.lock().expect("metrics lock poisoned").clone()
    }
}

/// The layer itself. See [`Metrics::layer`].
#[derive(Debug)]
pub struct MetricsLayer {
    metrics: Metrics,
}

/// Reads one field of an event. Structured fields arrive through a
/// visitor, one call per field, typed: `record_str`, `record_u64`, ...
struct OutcomeVisitor(Option<String>);

impl Visit for OutcomeVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "outcome" {
            self.0 = Some(value.to_string());
        }
    }

    // Every other type ends up here; we only care about `outcome`.
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "outcome" {
            self.0 = Some(format!("{value:?}"));
        }
    }
}

/// Stored in a span's "extensions" (a per-span type map) when it opens.
struct OpenedAt(Instant);

impl<S> Layer<S> for MetricsLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut outcome = OutcomeVisitor(None);
        event.record(&mut outcome);
        let mut m = self.metrics.0.lock().expect("metrics lock poisoned");
        *m.events
            .entry(event.metadata().level().to_string())
            .or_default() += 1;
        if let Some(outcome) = outcome.0 {
            *m.outcomes.entry(outcome).or_default() += 1;
        }
    }

    fn on_new_span(&self, _attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id)
            && span.name() == "job"
        {
            span.extensions_mut().insert(OpenedAt(Instant::now()));
        }
    }

    fn on_close(&self, id: span::Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(&id) else { return };
        if let Some(opened) = span.extensions().get::<OpenedAt>() {
            let mut m = self.metrics.0.lock().expect("metrics lock poisoned");
            m.jobs_finished += 1;
            m.job_time += opened.0.elapsed();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::prelude::*;

    #[test]
    fn counts_events_outcomes_and_job_spans() {
        let metrics = Metrics::default();
        let subscriber = tracing_subscriber::registry().with(metrics.layer());
        tracing::subscriber::with_default(subscriber, || {
            let _job = tracing::info_span!("job", job.id = 1).entered();
            tracing::info!(outcome = "succeeded", "done");
            tracing::warn!("careful");
            tracing::warn!(outcome = "failed", "oops");
            let _other = tracing::info_span!("not_a_job").entered();
        });
        let m = metrics.snapshot();
        assert_eq!(m.events["INFO"], 1);
        assert_eq!(m.events["WARN"], 2);
        assert_eq!(m.outcomes["succeeded"], 1);
        assert_eq!(m.outcomes["failed"], 1);
        assert_eq!(m.jobs_finished, 1, "only spans named `job` are timed");
    }
}
