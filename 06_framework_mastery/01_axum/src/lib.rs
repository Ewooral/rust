//! Mini project: an in-memory **notes JSON API** built with axum.
//!
//! - [`notes`]: the JSON shapes (serde), validation rules and the store;
//! - [`error`]: `ApiError`, one error type that implements `IntoResponse`;
//! - [`routes`]: shared state, handlers, nested routers and middleware
//!   (tracing, a request-id, CORS, timeouts, a body-size limit);
//! - [`client`]: a tiny raw-TCP HTTP client for the examples and tests.
//!
//! `tests/api.rs` drives the router with `tower::ServiceExt::oneshot` (no
//! network); `tests/e2e.rs` runs the real server on `127.0.0.1:0`.

pub mod client;
pub mod error;
pub mod notes;
pub mod routes;

pub use error::ApiError;
pub use notes::{ListParams, NewNote, Note, NoteStore, UpdateNote};
pub use routes::{AppState, RequestId, app};

use std::future::Future;
use tokio::net::TcpListener;

/// Serves the app on `listener` until `shutdown` completes, then stops
/// accepting connections and waits for requests in progress to finish.
pub async fn serve(
    listener: TcpListener,
    state: AppState,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> std::io::Result<()> {
    axum::serve(listener, app(state))
        .with_graceful_shutdown(shutdown)
        .await
}

/// Prints `tracing` events to the terminal. `RUST_LOG` picks how much,
/// e.g. `RUST_LOG=tower_http=debug`; the default shows this crate's logs at
/// `debug` and one line per request from `TraceLayer`.
pub fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("axum_basics=debug,tower_http=debug"));
    // `try_init` so calling it twice (as tests might) is harmless.
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
