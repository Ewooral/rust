//! The book-review API server.
//!
//! Run:  cp 06_framework_mastery/07_capstone_rest_api/.env.example .env.local   (then edit)
//!       APP_CONFIG_FILE=.env.local cargo run -p capstone_rest_api
//! or set the variables yourself (see `.env.example`).
//!
//! `capstone_rest_api healthcheck` asks a running server for `/health` and
//! exits 0 if it is healthy: the Docker image uses it as its `HEALTHCHECK`.

use std::process::ExitCode;

use capstone_rest_api::{
    AppState, Config, build_router, client, serve, shutdown_signal, telemetry,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> ExitCode {
    let config = match Config::load() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("configuration error: {err}");
            return ExitCode::from(2);
        }
    };

    if std::env::args().nth(1).as_deref() == Some("healthcheck") {
        return healthcheck(&config).await;
    }

    telemetry::init(config.log_format);
    match run(config).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            tracing::error!(%err, "the server stopped with an error");
            ExitCode::FAILURE
        }
    }
}

async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(?config, "starting");
    if config.jwt_secret.expose().contains("change-me") {
        tracing::warn!(
            "JWT_SECRET is the example value from .env.example: change it before deploying"
        );
    }
    let listener = TcpListener::bind(config.socket_addr()).await?;
    let app = build_router(AppState::build(config).await?);
    let addr = listener.local_addr()?;
    tracing::info!("listening on http://{addr}");
    serve(listener, app, shutdown_signal()).await?;
    tracing::info!("stopped");
    Ok(())
}

async fn healthcheck(config: &Config) -> ExitCode {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], config.port));
    match client::send(addr, "GET", "/health", None, None).await {
        Ok(response) if response.status == 200 => ExitCode::SUCCESS,
        Ok(response) => {
            eprintln!("unhealthy: {} {}", response.status, response.body);
            ExitCode::FAILURE
        }
        Err(err) => {
            eprintln!("unreachable: {err}");
            ExitCode::FAILURE
        }
    }
}
