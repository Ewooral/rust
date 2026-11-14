//! End to end: the real server on a real socket. Bind `127.0.0.1:0` (any
//! free port), serve in a background task, talk to it over TCP, then shut
//! it down gracefully and check that `serve` returns.
//!
//! Run: cargo test -p axum_basics --test e2e

use axum_basics::{AppState, client, serve};
use serde_json::{Value, json};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

#[tokio::test]
async fn notes_api_over_tcp_then_graceful_shutdown() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (stop_tx, stop_rx) = oneshot::channel::<()>();
    let server = tokio::spawn(serve(listener, AppState::default(), async {
        stop_rx.await.ok();
    }));

    let hello = client::get(addr, "/").await.unwrap();
    assert_eq!(
        (hello.status, hello.body.as_str()),
        (200, "Hello from Axum!")
    );

    let body = json!({ "title": "Ship it", "tags": ["release"] }).to_string();
    let created = client::send(addr, "POST", "/api/notes", Some(&body))
        .await
        .unwrap();
    assert_eq!(created.status, 201, "{}", created.body);
    assert_eq!(created.header("location"), Some("/api/notes/1"));
    assert!(created.header("x-request-id").is_some());

    let listed = client::get(addr, "/api/notes?tag=release").await.unwrap();
    let notes: Value = serde_json::from_str(&listed.body).unwrap();
    assert_eq!(notes[0]["title"], "Ship it");

    let missing = client::get(addr, "/api/notes/42").await.unwrap();
    assert_eq!(missing.status, 404);
    assert_eq!(missing.header("content-type"), Some("application/json"));

    stop_tx.send(()).unwrap();
    let finished = tokio::time::timeout(Duration::from_secs(5), server)
        .await
        .expect("server should stop within 5 s of the shutdown signal");
    finished.unwrap().unwrap();
}
