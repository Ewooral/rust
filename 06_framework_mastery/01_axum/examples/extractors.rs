//! Extractors: `Path`, `Query`, `Json`, `HeaderMap`, `State`, and what
//! axum answers when one of them can't make sense of the request (a
//! "rejection").
//! Run: cargo run -p axum_basics --example extractors

use axum::body::{Body, to_bytes};
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, Request, header};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tower::ServiceExt;

// Path: one capture, or several as a tuple (in the order they appear).
async fn repo(Path((owner, name)): Path<(String, String)>) -> String {
    format!("repo {name} owned by {owner}")
}

// Query: the `?a=1&b=2` part, deserialized into a struct with serde.
// `Option` fields may be missing; `#[serde(default)]` fills in a default.
#[derive(Deserialize)]
struct Search {
    q: String,
    #[serde(default = "first_page")]
    page: u32,
    lang: Option<String>,
}

fn first_page() -> u32 {
    1
}

async fn search(Query(search): Query<Search>) -> String {
    let lang = search.lang.as_deref().unwrap_or("any");
    format!(
        "searching {:?}, page {}, language {lang}",
        search.q, search.page
    )
}

// Json: the request body, deserialized. It also checks the request said
// `Content-Type: application/json`.
#[derive(Deserialize)]
struct SignUp {
    name: String,
    age: u8,
}

#[derive(Serialize)]
struct Welcome {
    message: String,
    adult: bool,
}

async fn sign_up(Json(form): Json<SignUp>) -> Json<Welcome> {
    Json(Welcome {
        message: format!("welcome, {}", form.name),
        adult: form.age >= 18,
    })
}

// Headers: `HeaderMap` gives all of them (a single header can also be read
// from `HeaderMap` with `.get(header::USER_AGENT)`).
async fn agent(headers: HeaderMap) -> String {
    let agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    format!("you are using {agent}; {} headers in total", headers.len())
}

// State: shared data given to the router with `.with_state(...)`.
#[derive(Clone)]
struct Config {
    site_name: &'static str,
}

// Several extractors at once. Order matters for only one rule: the one
// that reads the BODY (here `Json`) must come last.
async fn comment(
    State(config): State<Config>,
    Path(post_id): Path<u32>,
    headers: HeaderMap,
    Json(text): Json<String>,
) -> String {
    let who = headers
        .get("x-user")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anonymous");
    format!(
        "{}: {who} commented {text:?} on post {post_id}",
        config.site_name
    )
}

fn app() -> Router {
    Router::new()
        .route("/repos/{owner}/{name}", get(repo))
        .route("/search", get(search))
        .route("/sign-up", post(sign_up))
        .route("/agent", get(agent))
        .route("/posts/{id}/comments", post(comment))
        .with_state(Config {
            site_name: "MyBlog",
        })
}

async fn send(app: &Router, request: Request<Body>) {
    let line = format!("{} {}", request.method(), request.uri());
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    println!("{line}\n    -> {status} {}", String::from_utf8_lossy(&body));
}

fn get_req(uri: &str) -> Request<Body> {
    Request::get(uri)
        .header(header::USER_AGENT, "curl/8.5.0")
        .body(Body::empty())
        .unwrap()
}

fn json_req(uri: &str, json: &str) -> Request<Body> {
    Request::post(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-user", "ada")
        .body(Body::from(json.to_string()))
        .unwrap()
}

#[tokio::main]
async fn main() {
    let app = app();
    println!("--- extractors that succeed ---");
    send(&app, get_req("/repos/tokio-rs/axum")).await;
    send(&app, get_req("/search?q=rust")).await;
    send(&app, get_req("/search?q=rust&page=3&lang=en")).await;
    send(&app, json_req("/sign-up", r#"{"name":"Ada","age":36}"#)).await;
    send(&app, get_req("/agent")).await;
    send(&app, json_req("/posts/7/comments", r#""Nice post!""#)).await;

    println!("\n--- rejections: the handler never runs ---");
    send(&app, get_req("/search?page=2")).await; // `q` is missing
    send(&app, get_req("/search?q=rust&page=two")).await;
    let no_content_type = Request::post("/sign-up")
        .body(Body::from(r#"{"name":"Ada","age":36}"#))
        .unwrap();
    send(&app, no_content_type).await;
    send(&app, json_req("/sign-up", r#"{"name":"Ada""#)).await; // cut short
    send(&app, json_req("/sign-up", r#"{"name":"Ada"}"#)).await; // no age
    send(&app, json_req("/sign-up", r#"{"name":"Ada","age":300}"#)).await;
}
