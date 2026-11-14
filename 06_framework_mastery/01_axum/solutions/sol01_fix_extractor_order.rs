//! Solution to exercise 1: make it compile (E0277: "the trait bound `fn(...) {add_comment}:
//! Handler<_, _>` is not satisfied").
//!
//! `add_comment` takes three extractors, but in an order axum can't accept:
//! an extractor that reads the request BODY (`Json`, `String`, `Bytes`,
//! `Form`...) consumes it, so it must be the LAST argument. Everything else
//! (`State`, `Path`, `Query`, `HeaderMap`...) only looks at the request's
//! head and can go in any order before it.
//!
//! 1. Run it and read the error. It names the route, not the real problem.
//! 2. Put `#[axum::debug_handler]` on the line above `async fn add_comment`
//!    and run it again: the macro checks the handler itself and says
//!    exactly which argument is in the wrong place.
//! 3. Fix the order. (You may keep `#[axum::debug_handler]`: it changes
//!    nothing at run time.)
//!
//! Run: cargo run -p axum_basics --features solutions --bin sol01_fix_extractor_order

use axum::body::{Body, to_bytes};
use axum::extract::{Path, State};
use axum::http::{Request, header};
use axum::routing::post;
use axum::{Json, Router};
use tower::ServiceExt;

#[derive(Clone)]
struct AppState {
    site: &'static str,
}

// The body extractor (`Json`) moved to the end. `#[axum::debug_handler]`
// stays: it costs nothing at run time and gives readable errors next time.
#[axum::debug_handler]
async fn add_comment(
    State(state): State<AppState>,
    Path(post_id): Path<u32>,
    Json(text): Json<String>,
) -> String {
    format!("{}: comment {text:?} added to post {post_id}", state.site)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/posts/{id}/comments", post(add_comment))
        .with_state(AppState { site: "MyBlog" });

    let request = Request::post("/posts/7/comments")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#""Great post!""#))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    println!("{status} {}", String::from_utf8_lossy(&body));
}
