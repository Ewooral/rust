//! Responses: anything that implements `IntoResponse` can be returned from a
//! handler. Strings, status codes, JSON, HTML, headers, tuples of those,
//! redirects, `Result`, and a hand-built `Response`.
//! Run: cargo run -p axum_basics --example responses

use axum::Json;
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode, header};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use serde_json::json;
use tower::ServiceExt;

async fn text() -> &'static str {
    "plain text" // 200, text/plain; charset=utf-8
}

async fn status_only() -> StatusCode {
    StatusCode::NO_CONTENT // 204, no body
}

async fn json_body() -> Json<serde_json::Value> {
    Json(json!({ "id": 1, "tags": ["rust", "web"] })) // application/json
}

async fn html() -> Html<&'static str> {
    Html("<h1>Hello</h1>") // text/html; charset=utf-8
}

// A tuple: (status, headers..., body). The parts are applied in order.
async fn created() -> impl IntoResponse {
    (
        StatusCode::CREATED,
        [
            (header::LOCATION, "/items/7"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        Json(json!({ "id": 7 })),
    )
}

// Headers can also come from a `HeaderMap` built at run time.
async fn with_header_map() -> (HeaderMap, &'static str) {
    let mut headers = HeaderMap::new();
    headers.insert("x-powered-by", HeaderValue::from_static("axum"));
    (headers, "see the x-powered-by header")
}

async fn moved() -> Redirect {
    Redirect::permanent("/text") // 308 + Location
}

// Different branches return different types, so convert each one with
// `.into_response()`: then both are a plain `Response`.
async fn maybe(flag: bool) -> Response {
    if flag {
        Json(json!({ "ok": true })).into_response()
    } else {
        (StatusCode::BAD_REQUEST, "flag was false").into_response()
    }
}

// `Result<T, E>` works when both T and E implement IntoResponse.
async fn fallible() -> Result<&'static str, (StatusCode, String)> {
    Err((StatusCode::IM_A_TEAPOT, "no coffee here".to_string()))
}

// The low-level way: build the response yourself.
async fn by_hand() -> Response {
    Response::builder()
        .status(StatusCode::ACCEPTED)
        .header(header::CONTENT_TYPE, "text/csv")
        .body(Body::from("id,title\n1,Learn axum\n"))
        .unwrap()
}

fn app() -> Router {
    Router::new()
        .route("/text", get(text))
        .route("/status", get(status_only))
        .route("/json", get(json_body))
        .route("/html", get(html))
        .route("/created", get(created))
        .route("/header-map", get(with_header_map))
        .route("/old-text", get(moved))
        .route("/maybe-yes", get(|| maybe(true)))
        .route("/maybe-no", get(|| maybe(false)))
        .route("/fallible", get(fallible))
        .route("/by-hand", get(by_hand))
}

#[tokio::main]
async fn main() {
    let app = app();
    for uri in [
        "/text",
        "/status",
        "/json",
        "/html",
        "/created",
        "/header-map",
        "/old-text",
        "/maybe-yes",
        "/maybe-no",
        "/fallible",
        "/by-hand",
    ] {
        let request = Request::get(uri).body(Body::empty()).unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        println!("GET {uri} -> {}", response.status());
        for (name, value) in response.headers() {
            println!("    {name}: {}", value.to_str().unwrap());
        }
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        if !body.is_empty() {
            println!("    body: {:?}", String::from_utf8_lossy(&body));
        }
    }
}
