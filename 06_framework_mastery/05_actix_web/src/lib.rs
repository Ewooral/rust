//! # A notes JSON API on Actix-web
//!
//! The same API as the Axum lesson's mini project (06/01), so you can compare
//! the two frameworks line by line:
//!
//! - [`notes`]: JSON shapes, validation and the store (no HTTP in it at all);
//! - [`error`]: `ApiError`, which implements Actix's `ResponseError`;
//! - [`routes`]: `AppState`, the `#[get]`/`#[post]` handlers, `configure`,
//!   and [`app`], which adds the Logger and request-id middleware;
//! - [`client`]: a tiny raw-TCP HTTP client for the examples and tests.
//!
//! `tests/api.rs` calls the app with `actix_web::test` (no network);
//! `tests/e2e.rs` runs the real server on `127.0.0.1:0`.

pub mod client;
pub mod error;
pub mod notes;
pub mod routes;

pub use error::ApiError;
pub use notes::{ListParams, NewNote, Note, NoteStore, UpdateNote};
pub use routes::{AppState, RequestId, app, configure};

use actix_web::dev::Server;
use actix_web::{HttpServer, web};

/// Builds a running server on an already-bound listener (bind `127.0.0.1:0`
/// in tests to get a free port). Await the returned [`Server`] to run until it
/// stops; `server.handle().stop(true)` stops it gracefully from elsewhere.
pub fn serve(
    listener: std::net::TcpListener,
    state: web::Data<AppState>,
    workers: usize,
) -> std::io::Result<Server> {
    let server = HttpServer::new(move || app(state.clone()))
        .workers(workers)
        .shutdown_timeout(5) // seconds to finish requests in flight (default 30)
        .listen(listener)?
        .run();
    Ok(server)
}

/// Prints log lines. Actix logs through the `log` crate; `tracing-subscriber`
/// collects those too (its `tracing-log` feature is on by default).
/// `RUST_LOG` chooses how much, e.g. `RUST_LOG=actix_web=debug`.
pub fn init_logging() {
    use std::io::IsTerminal;
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("actix_web=info,actix_server=info,actix_web_basics=info")
    });
    // Colours only in a terminal; `try_init`, so a second call is harmless.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(std::io::stdout().is_terminal())
        .try_init();
}
