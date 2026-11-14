//! Shared state: one value every handler can reach, through `State`.
//! Run: cargo run -p axum_basics --example shared_state
//!
//! axum clones the state for every request, so the state should be cheap to
//! clone and should SHARE its data: put the data behind an `Arc`. Mutable
//! data also needs a lock (`Mutex`/`RwLock`) or an atomic.
//! `FromRef` lets a handler ask for just one piece of the state.

use axum::body::{Body, to_bytes};
use axum::extract::{FromRef, Path, State};
use axum::http::Request;
use axum::routing::{get, post};
use axum::{Json, Router};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::task::JoinSet;
use tower::ServiceExt;

#[derive(Clone)]
struct AppState {
    hits: Arc<AtomicU64>,                    // a counter: an atomic is enough
    votes: Arc<Mutex<HashMap<String, u32>>>, // a map: needs a lock
    config: Arc<Config>,                     // read-only: no lock at all
}

struct Config {
    poll_name: String,
}

// `FromRef` lets a handler extract `State<Arc<Config>>` from an `AppState`.
impl FromRef<AppState> for Arc<Config> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.config)
    }
}

async fn hit(State(state): State<AppState>) -> String {
    let n = state.hits.fetch_add(1, Ordering::Relaxed) + 1;
    format!("hit number {n}")
}

async fn vote(State(state): State<AppState>, Path(choice): Path<String>) -> String {
    // A std Mutex is fine in async code as long as the guard is dropped
    // before any `.await`. Here there is no `.await` at all.
    let mut votes = state.votes.lock().unwrap();
    let count = votes.entry(choice.clone()).or_insert(0);
    *count += 1;
    format!("{choice} now has {count}")
}

async fn results(
    State(config): State<Arc<Config>>, // just the piece we need, via FromRef
    State(state): State<AppState>,     // or the whole thing
) -> Json<serde_json::Value> {
    let votes = state.votes.lock().unwrap().clone();
    Json(serde_json::json!({ "poll": config.poll_name, "votes": votes }))
}

async fn get_text(app: &Router, method: &str, uri: &str) -> String {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(body.to_vec()).unwrap()
}

#[tokio::main]
async fn main() {
    let state = AppState {
        hits: Arc::new(AtomicU64::new(0)),
        votes: Arc::new(Mutex::new(HashMap::new())),
        config: Arc::new(Config {
            poll_name: "Best pet".to_string(),
        }),
    };
    let app = Router::new()
        .route("/hit", get(hit))
        .route("/vote/{choice}", post(vote))
        .route("/results", get(results))
        .with_state(state.clone()); // `state` is still ours: it's just Arcs

    // 100 requests at the same time, each on its own task.
    let mut tasks = JoinSet::new();
    for _ in 0..100 {
        let app = app.clone();
        tasks.spawn(async move { get_text(&app, "GET", "/hit").await });
    }
    tasks.join_all().await;
    println!(
        "after 100 concurrent requests the counter is {}",
        state.hits.load(Ordering::Relaxed)
    );
    println!("one more: {}", get_text(&app, "GET", "/hit").await);

    for choice in ["cat", "dog", "cat"] {
        println!(
            "{}",
            get_text(&app, "POST", &format!("/vote/{choice}")).await
        );
    }
    println!("{}", get_text(&app, "GET", "/results").await);
}
