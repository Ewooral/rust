//! Mini project: an in-memory notes JSON API with axum.
//!
//! This folder started as a one-route "Hello from Axum!" server on
//! 127.0.0.1:3000 (now `examples/hello.rs`). The address and that route are
//! kept: `GET /` still says hello, and the notes API lives under `/api`.
//!
//!   cargo run -p axum_basics                     serve on 127.0.0.1:3000
//!   cargo run -p axum_basics -- 127.0.0.1:8080   serve somewhere else
//!   cargo test -p axum_basics
//!
//! Then, in another terminal:
//!   curl -i http://127.0.0.1:3000/
//!   curl -i -X POST http://127.0.0.1:3000/api/notes -H "content-type: application/json" -d '{"title":"Learn axum","tags":["rust"]}'
//!   curl -i http://127.0.0.1:3000/api/notes?tag=rust
//!
//! Stop it with Ctrl+C: the server finishes requests in progress first.

use axum_basics::{AppState, init_tracing, serve};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    init_tracing();
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:3000".to_string());

    let listener = TcpListener::bind(&addr).await.unwrap_or_else(|e| {
        eprintln!("could not listen on {addr}: {e}");
        std::process::exit(1);
    });
    println!("listening on http://{}", listener.local_addr().unwrap());

    if let Err(e) = serve(listener, AppState::default(), shutdown_signal()).await {
        eprintln!("server error: {e}");
        std::process::exit(1);
    }
    println!("server stopped");
}

/// Completes when the user presses Ctrl+C.
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for Ctrl+C");
    println!("Ctrl+C received: finishing requests in progress, then stopping");
}
