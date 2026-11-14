//! Error handling: a custom error type that implements `IntoResponse`, so
//! handlers can return `Result<T, AppError>` and use `?` on anything that
//! converts into an `AppError`.
//! Run: cargo run -p axum_basics --example error_handling
//!
//! The rule of thumb: errors the CLIENT caused (bad input, missing thing)
//! get a clear 4xx message; errors WE caused (a file we can't read, a bug)
//! get a generic 500 for the client, and the details go to our logs only.

use axum::body::{Body, to_bytes};
use axum::extract::Path;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use std::num::ParseIntError;
use tower::ServiceExt;

#[derive(Debug)]
enum AppError {
    NotFound(String),
    BadInput(String),
    Internal(String), // details for the log, never for the client
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(what) => (StatusCode::NOT_FOUND, format!("{what} not found")),
            AppError::BadInput(why) => (StatusCode::BAD_REQUEST, why),
            AppError::Internal(details) => {
                eprintln!("    [log] internal error: {details}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "something went wrong".to_string(),
                )
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

// `From` impls make `?` work: the error converts automatically.
impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError::BadInput(format!("not a number: {e}"))
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

async fn square(Path(raw): Path<String>) -> Result<String, AppError> {
    let n: i64 = raw.parse()?; // ParseIntError -> AppError::BadInput
    let squared = n
        .checked_mul(n)
        .ok_or_else(|| AppError::BadInput(format!("{n} squared is too big")))?;
    Ok(format!("{n} squared is {squared}"))
}

async fn user(Path(id): Path<u32>) -> Result<Json<serde_json::Value>, AppError> {
    match id {
        1 => Ok(Json(json!({ "id": 1, "name": "Ada" }))),
        _ => Err(AppError::NotFound(format!("user {id}"))),
    }
}

async fn report() -> Result<String, AppError> {
    let text = tokio::fs::read_to_string("no/such/report.txt").await?; // io::Error -> Internal
    Ok(text)
}

async fn call(app: &Router, uri: &str) {
    let request = Request::get(uri).body(Body::empty()).unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    println!(
        "GET {uri:22} -> {status} {}",
        String::from_utf8_lossy(&body)
    );
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/square/{n}", get(square))
        .route("/users/{id}", get(user))
        .route("/report", get(report));

    call(&app, "/square/12").await;
    call(&app, "/square/twelve").await;
    call(&app, "/square/9999999999").await;
    call(&app, "/users/1").await;
    call(&app, "/users/2").await;
    call(&app, "/report").await;
}
