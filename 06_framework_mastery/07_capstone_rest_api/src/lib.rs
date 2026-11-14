//! # The capstone: a book-review REST API
//!
//! Axum + SQLx (SQLite) + Tokio, in layers. A request travels down and the
//! answer comes back up:
//!
//! ```text
//! routes      which handler, and the middleware around it   (src/routes.rs)
//!   handlers  HTTP in, HTTP out; one service call each        (src/handlers/)
//!     service the rules: validation, who may do what         (src/service/)
//!       repository  the only code that writes SQL           (src/repository/)
//!         SQLite    migrations in migrations/
//! ```
//!
//! Alongside: [`config`] (env + file), [`auth`] (Argon2 passwords, JWT
//! tokens, the `AuthUser` extractor), [`domain`] (types, validation,
//! pagination), [`error`] (one JSON error shape), [`telemetry`] (tracing).

pub mod auth;
pub mod client;
pub mod config;
pub mod domain;
pub mod error;
pub mod handlers;
pub mod repository;
pub mod routes;
pub mod service;
pub mod state;
pub mod telemetry;

pub use config::Config;
pub use error::AppError;
pub use routes::build_router;
pub use state::{AppState, StartupError};

use std::future::Future;

use tokio::net::TcpListener;

/// Builds the state (pool, migrations, services) and the router in one go.
pub async fn build_app(config: Config) -> Result<axum::Router, StartupError> {
    Ok(build_router(AppState::build(config).await?))
}

/// Serves `app` until `shutdown` completes, then stops accepting connections
/// and lets requests in flight finish (graceful shutdown).
pub async fn serve(
    listener: TcpListener,
    app: axum::Router,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> std::io::Result<()> {
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
}

/// Completes on Ctrl+C, or on SIGTERM on Unix: what `docker stop` and
/// Kubernetes send before they kill a container.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        if tokio::signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => tracing::info!("Ctrl+C received, shutting down"),
        _ = terminate => tracing::info!("SIGTERM received, shutting down"),
    }
}
