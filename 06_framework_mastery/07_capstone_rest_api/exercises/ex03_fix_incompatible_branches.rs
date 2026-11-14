//! Exercise 3: make it compile (E0308): two kinds of response from one handler.
//!
//! `find_book` answers 200 with JSON when the book exists, and a bare 404
//! when it doesn't. `-> impl IntoResponse` means "ONE type that implements
//! IntoResponse", but the two branches have two different types.
//!
//! Fix it the usual axum way: turn each branch into a `Response` with
//! `.into_response()` and declare the return type as `Response`. (Returning
//! `Result<Json<Book>, StatusCode>` is another good fix; try both.)
//!
//! The compiler also suggests `Box<dyn IntoResponse>`: a dead end. A box
//! isn't a response itself, so the next error is E0277 "the trait bound
//! `fn(..) {find_book}: Handler<_, _>` is not satisfied".
//!
//! Run: cargo run -p capstone_rest_api --features exercises --bin ex03_fix_incompatible_branches

use axum::Json;
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::Path;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde_json::{Value, json};
use tower::ServiceExt;

async fn find_book(Path(id): Path<u32>) -> impl IntoResponse {
    if id == 1 {
        (StatusCode::OK, Json(json!({ "id": 1, "title": "Emma" })))
    } else {
        StatusCode::NOT_FOUND
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/books/{id}", get(find_book));
    for path in ["/books/1", "/books/2"] {
        let request = Request::get(path).body(Body::empty()).unwrap();
        let response: Response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
        println!("GET {path} -> {status} {json}");
    }
}
