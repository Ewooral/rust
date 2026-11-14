//! Routing: paths, methods, captures, wildcards, nested and merged routers,
//! and what happens when nothing matches (404) or the method is wrong (405).
//! Run: cargo run -p axum_basics --example routing
//!
//! No network here: `tower::ServiceExt::oneshot` hands one request straight
//! to the router and gives back the response, exactly as the server would.

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::Path;
use axum::http::{Request, StatusCode};
use axum::routing::{get, post};
use tower::ServiceExt; // for `oneshot`

// A handler is just an async function whose arguments are extractors and
// whose return value can become a response.
async fn root() -> &'static str {
    "home"
}

async fn list_users() -> &'static str {
    "all users"
}

async fn create_user() -> (StatusCode, &'static str) {
    (StatusCode::CREATED, "user created")
}

async fn show_user(Path(id): Path<u32>) -> String {
    format!("user {id}")
}

async fn show_file(Path(path): Path<String>) -> String {
    format!("file {path:?}")
}

fn users() -> Router {
    Router::new()
        // One path, two methods: chain them on the same `route`.
        .route("/", get(list_users).post(create_user))
        // `{id}` captures one segment (axum 0.8; older versions used `:id`).
        .route("/{id}", get(show_user))
}

fn misc() -> Router {
    Router::new()
        // `{*path}` captures the rest of the path, slashes included.
        .route("/files/{*path}", get(show_file))
        .route("/ping", post(|| async { "pong" }))
}

fn app() -> Router {
    Router::new()
        .route("/", get(root))
        .nest("/users", users()) // every route of `users()` gets the prefix /users
        .merge(misc()) // routes of `misc()` are added as they are
        .fallback(|| async { (StatusCode::NOT_FOUND, "nothing here") })
}

async fn call(app: &Router, method: &str, uri: &str) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let allow = response
        .headers()
        .get("allow")
        .map(|v| format!("  (allow: {})", v.to_str().unwrap()))
        .unwrap_or_default();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    println!(
        "{method:6} {uri:22} -> {status} {:?}{allow}",
        String::from_utf8_lossy(&body)
    );
}

#[tokio::main]
async fn main() {
    let app = app();
    call(&app, "GET", "/").await;
    call(&app, "GET", "/users").await;
    call(&app, "POST", "/users").await;
    call(&app, "GET", "/users/42").await;
    call(&app, "GET", "/users/abc").await; // capture can't parse as u32
    call(&app, "GET", "/files/docs/notes.txt").await;
    call(&app, "POST", "/ping").await;
    call(&app, "GET", "/ping").await; // right path, wrong method
    call(&app, "DELETE", "/users/42").await;
    call(&app, "GET", "/nope").await; // the fallback
}
