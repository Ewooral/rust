//! Exercise 2: make it compile (E0277 on the route again; the real cause,
//! "future cannot be sent between threads safely", has no error code).
//!
//! axum runs handlers on Tokio's multi-threaded runtime, so a handler's
//! future must be `Send`: after any `.await` it may continue on another
//! thread. `record_visit` keeps a `std::sync::MutexGuard` alive ACROSS an
//! `.await`, and a std guard must be unlocked by the thread that locked it,
//! so it isn't `Send`.
//!
//! 1. Run it: the error points at `get(record_visit)` and says the function
//!    isn't a `Handler`. Not very helpful.
//! 2. Add `#[axum::debug_handler]` above the handler and run it again: now
//!    the error names the guard and the `.await`.
//! 3. Fix it WITHOUT changing the type of `visits`: finish with the lock
//!    (copy the number out) in a block of its own, so the guard is dropped
//!    BEFORE the `.await`. (Switching to `tokio::sync::Mutex` also works,
//!    but it's slower and only needed when you must hold a lock while you
//!    wait.)
//!
//! Run: cargo run -p axum_basics --features exercises --bin ex02_fix_guard_across_await

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::State;
use axum::http::Request;
use axum::routing::get;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tower::ServiceExt;

#[derive(Clone, Default)]
struct AppState {
    visits: Arc<Mutex<u64>>,
}

/// Pretends to write the count to a database.
async fn save(count: u64) {
    tokio::time::sleep(Duration::from_millis(5)).await;
    let _ = count;
}

async fn record_visit(State(state): State<AppState>) -> String {
    let mut visits = state.visits.lock().unwrap();
    *visits += 1;
    save(*visits).await;
    format!("you are visitor number {}", *visits)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(record_visit))
        .with_state(AppState::default());

    for _ in 0..3 {
        let request = Request::get("/").body(Body::empty()).unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        println!("{}", String::from_utf8_lossy(&body));
    }
}
