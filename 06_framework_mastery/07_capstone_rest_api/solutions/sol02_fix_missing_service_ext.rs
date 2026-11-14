//! Solution to exercise 2: make it compile (E0599): call a `Router` like a function.
//!
//! Handler tests send a request straight into the `Router` with `oneshot`,
//! no network needed. But `oneshot` isn't a method of `Router` itself: it is
//! provided by an extension TRAIT, and a trait's methods can only be called
//! when the trait is in scope.
//!
//! Read the error (it names the trait and the crate), add the one missing
//! `use` line, and run it.
//!
//! Answer: `use tower::ServiceExt;`. A `Router` is a Tower `Service`, and
//! `ServiceExt` adds `oneshot` to every service.
//!
//! Run: cargo run -p capstone_rest_api --features solutions --bin sol02_fix_missing_service_ext

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use axum::routing::get;
use tower::ServiceExt; // brings `oneshot` (and `ready`, `call`...) into scope

async fn ping() -> &'static str {
    "pong"
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/ping", get(ping));

    let request = Request::get("/ping").body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024).await.unwrap();
    println!("GET /ping -> {}", String::from_utf8_lossy(&body));
}
