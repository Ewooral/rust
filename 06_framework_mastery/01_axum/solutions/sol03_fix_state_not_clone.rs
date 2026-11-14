//! Solution to exercise 3: make it compile (E0277: the trait bound `AppState: Clone` is
//! not satisfied).
//!
//! axum hands every request its own copy of the state, so the state type
//! must implement `Clone` (and `Send + Sync + 'static`). `AppState` forgot
//! the derive.
//!
//! 1. Run it and find the line the compiler blames.
//! 2. Add the missing derive.
//! 3. Then make sure the clone is CHEAP and SHARED: cloning a struct that
//!    holds a big `Vec` copies the whole list for every request, and changes
//!    made through one copy are invisible to the others. Put the list behind
//!    an `Arc` (it is only read, so no lock is needed), so that a clone is
//!    just one more pointer to the same list.
//!
//! Run: cargo run -p axum_basics --features solutions --bin sol03_fix_state_not_clone

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::{Path, State};
use axum::http::Request;
use axum::routing::get;
use std::sync::Arc;
use tower::ServiceExt;

// `Clone` because axum clones the state for each request; `Arc` so that a
// clone is one more pointer to the SAME list, not a copy of it.
#[derive(Clone)]
struct AppState {
    quotes: Arc<Vec<String>>,
}

async fn quote(State(state): State<AppState>, Path(n): Path<usize>) -> String {
    match state.quotes.get(n) {
        Some(q) => q.clone(),
        None => format!("only {} quotes", state.quotes.len()),
    }
}

#[tokio::main]
async fn main() {
    let state = AppState {
        quotes: Arc::new(vec![
            "Simple is better than complex.".to_string(),
            "Make it work, make it right, make it fast.".to_string(),
        ]),
    };
    let app = Router::new()
        .route("/quotes/{n}", get(quote))
        .with_state(state);

    for uri in ["/quotes/0", "/quotes/1", "/quotes/5"] {
        let request = Request::get(uri).body(Body::empty()).unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        println!("GET {uri} -> {}", String::from_utf8_lossy(&body));
    }
}
