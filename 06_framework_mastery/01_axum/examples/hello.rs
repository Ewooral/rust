//! The first axum program: one route, `GET /`, answering "Hello from Axum!".
//! Run: cargo run -p axum_basics --example hello
//!
//! This is the code this folder started with (it was `src/main.rs`). The
//! original listened on 127.0.0.1:3000 and served forever; that job now
//! belongs to the mini project (`cargo run -p axum_basics`), which keeps the
//! same address and the same `GET /` route. So that this example finishes on
//! its own, it differs in three small ways, each marked CHANGED:
//! it binds port 0, asks itself for `/` once, and then shuts down.

use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Hello from Axum!" }));

    // CHANGED: port 0 = "any free port", so it never clashes with a server
    // already running on 3000.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    println!("listening on http://{addr}");

    // CHANGED: stop when `stop_tx` sends, instead of never.
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                stop_rx.await.ok();
            })
            .await
            .unwrap();
    });

    // CHANGED: be our own browser for one request.
    let response = axum_basics::client::get(addr, "/").await.unwrap();
    println!("GET / -> {} {:?}", response.status, response.body);

    stop_tx.send(()).unwrap();
    server.await.unwrap();
    println!("server stopped");
}
