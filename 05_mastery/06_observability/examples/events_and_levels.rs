//! Events and levels: what gets printed, and who decides.
//!
//! Run: cargo run -p observability --example events_and_levels

use tracing::{Level, debug, error, info, trace, warn};

fn emit_all_levels() {
    trace!("trace: every little step");
    debug!("debug: details for the developer");
    info!("info: the normal story of the program");
    warn!("warn: something odd, but handled");
    error!("error: something failed");
}

fn expensive_summary() -> String {
    // Imagine this walks a big data structure.
    "42 items, 3 duplicates".to_string()
}

fn main() {
    // 1. No subscriber yet: events go nowhere. `tracing` only DESCRIBES what
    //    happens; a subscriber decides what to do with it.
    println!("--- no subscriber:");
    info!("nobody sees this");

    // 2. The fmt subscriber, at its default maximum level, INFO.
    //    `with_default` uses it only inside the closure (and this thread).
    let default = tracing_subscriber::fmt().without_time().finish();
    tracing::subscriber::with_default(default, || {
        println!("--- fmt() with its default max level (INFO):");
        emit_all_levels();
    });

    // 3. Everything, down to TRACE.
    let verbose = tracing_subscriber::fmt()
        .without_time()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::with_default(verbose, || {
        println!("--- with_max_level(Level::TRACE):");
        emit_all_levels();

        // 4. Skip expensive work when nobody would see the result.
        if tracing::enabled!(Level::DEBUG) {
            debug!(
                summary = expensive_summary(),
                "only computed when DEBUG is on"
            );
        }
    });
}
