//! Handler tests WITHOUT a network: build the router, hand it one request
//! with `tower::ServiceExt::oneshot`, and inspect the response. No port, no
//! socket, no waiting: each test is a plain function call.
//!
//! Run: cargo test -p axum_basics --test api

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use axum_basics::{AppState, app};
use serde_json::{Value, json};
use tower::ServiceExt; // for `oneshot`

struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: String,
}

impl Reply {
    fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap_or_else(|e| panic!("{e}: {:?}", self.body))
    }
}

async fn send(app: &Router, request: Request<Body>) -> Reply {
    // `oneshot` takes the service by value, so clone the router (cheap).
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    Reply {
        status,
        headers,
        body: String::from_utf8(bytes.to_vec()).unwrap(),
    }
}

fn get(uri: &str) -> Request<Body> {
    Request::get(uri).body(Body::empty()).unwrap()
}

fn with_json(method: Method, uri: &str, json: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json.to_string()))
        .unwrap()
}

async fn create(app: &Router, title: &str, tags: &[&str]) -> Value {
    let reply = send(
        app,
        with_json(
            Method::POST,
            "/api/notes",
            json!({ "title": title, "tags": tags }),
        ),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.body);
    reply.json()
}

#[tokio::test]
async fn hello_route_still_says_hello() {
    let reply = send(&app(AppState::default()), get("/")).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body, "Hello from Axum!");
}

#[tokio::test]
async fn create_returns_201_location_and_the_note() {
    let app = app(AppState::default());
    let request = with_json(
        Method::POST,
        "/api/notes",
        json!({ "title": "  Learn axum ", "body": "extractors", "tags": ["rust"] }),
    );
    let reply = send(&app, request).await;
    assert_eq!(reply.status, StatusCode::CREATED);
    assert_eq!(reply.headers[header::LOCATION], "/api/notes/1");
    assert_eq!(
        reply.json(),
        json!({ "id": 1, "title": "Learn axum", "body": "extractors", "tags": ["rust"], "done": false })
    );

    let fetched = send(&app, get("/api/notes/1")).await;
    assert_eq!(fetched.status, StatusCode::OK);
    assert_eq!(fetched.json(), reply.json());
}

#[tokio::test]
async fn invalid_notes_get_422_with_every_problem() {
    let app = app(AppState::default());
    let request = with_json(
        Method::POST,
        "/api/notes",
        json!({ "title": " ", "tags": ["Not OK"] }),
    );
    let reply = send(&app, request).await;
    assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        reply.json(),
        json!({
            "error": "validation failed",
            "details": [
                "title must not be empty",
                "tag \"Not OK\" must be 1 to 20 characters of a-z, 0-9 or -"
            ]
        })
    );
}

#[tokio::test]
async fn rejections_become_json_errors() {
    let app = app(AppState::default());

    let broken = Request::post("/api/notes")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{\"title\":"))
        .unwrap();
    let reply = send(&app, broken).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(reply.json()["error"].as_str().unwrap().contains("JSON"));

    let no_content_type = Request::post("/api/notes")
        .body(Body::from("{\"title\":\"x\"}"))
        .unwrap();
    let reply = send(&app, no_content_type).await;
    assert_eq!(reply.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(
        reply.json()["error"],
        "Expected request with `Content-Type: application/json`"
    );

    let unknown_field = with_json(
        Method::POST,
        "/api/notes",
        json!({ "title": "x", "colour": "red" }),
    );
    let reply = send(&app, unknown_field).await;
    assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        reply.body.contains("unknown field `colour`"),
        "{}",
        reply.body
    );

    let reply = send(&app, get("/api/notes/abc")).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        reply.json()["error"],
        "Invalid URL: Cannot parse `abc` to a `u64`"
    );

    let reply = send(&app, get("/api/notes?limit=lots")).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn missing_notes_and_routes_are_json_404s() {
    let app = app(AppState::default());
    let reply = send(&app, get("/api/notes/99")).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(reply.json(), json!({ "error": "note 99 not found" }));

    let reply = send(&app, get("/nope")).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(reply.json(), json!({ "error": "no route for GET /nope" }));
}

#[tokio::test]
async fn wrong_method_is_405() {
    let request = Request::put("/api/notes").body(Body::empty()).unwrap();
    let reply = send(&app(AppState::default()), request).await;
    assert_eq!(reply.status, StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn list_filters_and_paginates() {
    let app = app(AppState::default());
    create(&app, "one", &["rust"]).await;
    create(&app, "two", &[]).await;
    create(&app, "three", &["rust", "web"]).await;
    let done = with_json(Method::PATCH, "/api/notes/3", json!({ "done": true }));
    assert_eq!(send(&app, done).await.status, StatusCode::OK);

    let titles = |reply: Reply| -> Vec<String> {
        reply
            .json()
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n["title"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(
        titles(send(&app, get("/api/notes")).await),
        ["one", "two", "three"]
    );
    assert_eq!(
        titles(send(&app, get("/api/notes?tag=rust")).await),
        ["one", "three"]
    );
    assert_eq!(
        titles(send(&app, get("/api/notes?tag=rust&done=false")).await),
        ["one"]
    );
    assert_eq!(
        titles(send(&app, get("/api/notes?limit=1&offset=1")).await),
        ["two"]
    );
}

#[tokio::test]
async fn patch_changes_only_the_given_fields() {
    let app = app(AppState::default());
    create(&app, "draft", &["a"]).await;
    let change = with_json(
        Method::PATCH,
        "/api/notes/1",
        json!({ "title": "final", "done": true }),
    );
    let reply = send(&app, change).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(
        reply.json(),
        json!({ "id": 1, "title": "final", "body": "", "tags": ["a"], "done": true })
    );

    let bad = with_json(Method::PATCH, "/api/notes/1", json!({ "title": "" }));
    assert_eq!(
        send(&app, bad).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let missing = with_json(Method::PATCH, "/api/notes/9", json!({ "done": true }));
    assert_eq!(send(&app, missing).await.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_then_gone() {
    let app = app(AppState::default());
    create(&app, "temporary", &[]).await;
    let delete = || Request::delete("/api/notes/1").body(Body::empty()).unwrap();
    let reply = send(&app, delete()).await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    assert!(reply.body.is_empty());
    assert_eq!(send(&app, delete()).await.status, StatusCode::NOT_FOUND);
    assert_eq!(
        send(&app, get("/api/notes/1")).await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn state_is_shared_between_requests_and_with_the_test() {
    let state = AppState::default();
    let app = app(state.clone());
    create(&app, "a", &[]).await;
    create(&app, "b", &[]).await;
    assert_eq!(state.store.len(), 2); // the test sees what the handlers did
    let reply = send(&app, get("/health")).await;
    assert_eq!(reply.json(), json!({ "status": "ok", "notes": 2 }));
}

#[tokio::test]
async fn request_id_is_generated_or_echoed() {
    let app = app(AppState::default());
    let reply = send(&app, get("/")).await;
    let generated = reply.headers["x-request-id"].to_str().unwrap();
    assert!(generated.starts_with("req-"), "{generated}");

    let tagged = Request::get("/")
        .header("x-request-id", "abc-123")
        .body(Body::empty())
        .unwrap();
    let reply = send(&app, tagged).await;
    assert_eq!(reply.headers["x-request-id"], "abc-123");
}

#[tokio::test]
async fn cors_preflight_is_answered() {
    let preflight = Request::builder()
        .method(Method::OPTIONS)
        .uri("/api/notes")
        .header(header::ORIGIN, "http://localhost:5173")
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
        .body(Body::empty())
        .unwrap();
    let reply = send(&app(AppState::default()), preflight).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.headers[header::ACCESS_CONTROL_ALLOW_ORIGIN], "*");
    let methods = reply.headers[header::ACCESS_CONTROL_ALLOW_METHODS]
        .to_str()
        .unwrap();
    assert!(methods.contains("POST"), "{methods}");
}

#[tokio::test]
async fn bodies_over_the_limit_get_413() {
    let huge = "x".repeat(axum_basics::routes::BODY_LIMIT + 1);
    let request = with_json(
        Method::POST,
        "/api/notes",
        json!({ "title": "big", "body": huge }),
    );
    let reply = send(&app(AppState::default()), request).await;
    assert_eq!(reply.status, StatusCode::PAYLOAD_TOO_LARGE);
}
