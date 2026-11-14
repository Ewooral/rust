//! Solution to exercise 3: make the test pass. The spans are telling lies.
//!
//! `handle` enters a span with `span.enter()` and keeps the guard while it
//! awaits. That compiles (the guard from `enter()` is `Send`), but while the
//! task is paused the span STAYS entered on the thread, so whatever else
//! runs there meanwhile is logged inside it. Run the test: the requests end
//! up nested inside each other, and the bystander's line lands inside a
//! request it has nothing to do with.
//!
//! Never hold a span guard across `.await`. Either put `#[instrument]` on
//! `handle` (with the right span name), or wrap the async work and call
//! `.instrument(span)` on it (`use tracing::Instrument`). (Had the code used
//! `.entered()`, whose guard is not `Send`, `tokio::spawn` would have refused
//! it: "future cannot be sent between threads safely".)
//!
//! Test: cargo test -p observability --features solutions --bin sol03_fix_span_across_await

use std::time::Duration;

use tracing::{info, instrument};

// #[instrument] attaches the span to the FUTURE: it is entered each time
// the future runs and exited each time it pauses, on whichever thread.
// (The other fix: `async { ... }.instrument(info_span!(...)).await`.)
#[instrument(name = "request")]
async fn handle(request_id: u32) {
    tokio::time::sleep(Duration::from_millis(5)).await;
    info!("handled");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let task = tokio::spawn(handle(1));
    task.await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use observability::capture::CapturedLogs;

    #[tokio::test]
    async fn each_request_logs_inside_its_own_span() {
        let logs = CapturedLogs::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(logs.clone())
            .with_ansi(false)
            .without_time()
            .with_target(false)
            .finish();
        let _guard = tracing::subscriber::set_default(subscriber);

        let one = tokio::spawn(handle(1));
        let two = tokio::spawn(handle(2));
        let bystander = tokio::spawn(async {
            tokio::time::sleep(Duration::from_millis(2)).await;
            info!("not part of any request");
        });
        let _ = tokio::join!(one, two, bystander);

        let mut lines = logs.lines();
        lines.sort();
        assert_eq!(
            lines,
            [
                " INFO not part of any request",
                " INFO request{request_id=1}: handled",
                " INFO request{request_id=2}: handled",
            ]
        );
    }
}
