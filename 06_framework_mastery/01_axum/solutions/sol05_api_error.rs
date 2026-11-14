//! Solution to exercise 5: a custom error type that implements `IntoResponse`.
//!
//! The handlers below are finished: they return `Result<_, ApiError>` and
//! use `?`. What's missing is how an `ApiError` becomes a response.
//! Implement:
//!
//! - `IntoResponse for ApiError`. Every error answers with a JSON body:
//!   - `NotFound(what)`  -> 404, `{"error": "<what> not found"}`
//!   - `Invalid(list)`   -> 422, `{"error": "invalid input", "details": [..list..]}`
//!   - `Rejected(status, message)` -> that status, `{"error": "<message>"}`
//!   - `Internal(details)` -> 500, `{"error": "internal error"}`. The
//!     details are for the server's log (`eprintln!` them), NEVER for the
//!     client: they might reveal file paths, SQL or secrets.
//! - `From<JsonRejection> for ApiError`, so `payload?` works in a handler:
//!   keep the rejection's own status (`rejection.status()`) and message
//!   (`rejection.body_text()`).
//!
//! Hints: `(StatusCode, Json(serde_json::json!({...}))).into_response()`.
//!
//! Test: cargo test -p axum_basics --features solutions --bin sol05_api_error

use axum::extract::Path;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug)]
enum ApiError {
    NotFound(String),
    Invalid(Vec<String>),
    Rejected(StatusCode, String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            ApiError::NotFound(what) => (
                StatusCode::NOT_FOUND,
                json!({ "error": format!("{what} not found") }),
            ),
            ApiError::Invalid(details) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({ "error": "invalid input", "details": details }),
            ),
            ApiError::Rejected(status, message) => (status, json!({ "error": message })),
            ApiError::Internal(details) => {
                eprintln!("internal error: {details}"); // the log gets the details
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({ "error": "internal error" }), // the client doesn't
                )
            }
        };
        (status, Json(body)).into_response()
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        ApiError::Rejected(rejection.status(), rejection.body_text())
    }
}

// ----- finished handlers: no changes needed below this line ---------------

#[derive(Deserialize)]
struct Order {
    item: String,
    quantity: u32,
}

async fn place_order(payload: Result<Json<Order>, JsonRejection>) -> Result<String, ApiError> {
    let Json(order) = payload?; // JsonRejection -> ApiError, through From
    let mut problems = Vec::new();
    if order.item.trim().is_empty() {
        problems.push("item must not be empty".to_string());
    }
    if order.quantity == 0 || order.quantity > 10 {
        problems.push("quantity must be 1 to 10".to_string());
    }
    if !problems.is_empty() {
        return Err(ApiError::Invalid(problems));
    }
    Ok(format!("ordered {} x {}", order.quantity, order.item))
}

async fn product(Path(id): Path<u32>) -> Result<String, ApiError> {
    match id {
        1 => Ok("teapot".to_string()),
        13 => Err(ApiError::Internal(
            "database file C:\\data\\shop.db is locked".to_string(),
        )),
        _ => Err(ApiError::NotFound(format!("product {id}"))),
    }
}

fn app() -> Router {
    Router::new()
        .route("/orders", post(place_order))
        .route("/products/{id}", get(product))
}

#[tokio::main]
async fn main() {
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use tower::ServiceExt;

    for uri in ["/products/1", "/products/2", "/products/13"] {
        let request = Request::get(uri).body(Body::empty()).unwrap();
        let response = app().oneshot(request).await.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        println!("GET {uri} -> {status} {}", String::from_utf8_lossy(&body));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, header};
    use serde_json::Value;
    use tower::ServiceExt;

    async fn send(request: Request<Body>) -> (StatusCode, Value) {
        let response = app().oneshot(request).await.unwrap();
        let status = response.status();
        assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&body).unwrap())
    }

    fn order(json: &str) -> Request<Body> {
        Request::post("/orders")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json.to_string()))
            .unwrap()
    }

    #[tokio::test]
    async fn not_found_and_internal_errors() {
        let get = |uri: &str| Request::get(uri).body(Body::empty()).unwrap();
        assert_eq!(
            send(get("/products/2")).await,
            (
                StatusCode::NOT_FOUND,
                json!({ "error": "product 2 not found" })
            )
        );
        let (status, body) = send(get("/products/13")).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body, json!({ "error": "internal error" }));
        assert!(!body.to_string().contains("shop.db"), "details leaked");
    }

    #[tokio::test]
    async fn validation_lists_every_problem() {
        assert_eq!(
            send(order(r#"{"item":" ","quantity":0}"#)).await,
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({
                    "error": "invalid input",
                    "details": ["item must not be empty", "quantity must be 1 to 10"]
                })
            )
        );
    }

    #[tokio::test]
    async fn rejections_keep_their_status() {
        let (status, body) = send(order(r#"{"item":"tea""#)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body["error"].as_str().unwrap().contains("JSON"), "{body}");

        let no_content_type = Request::post("/orders")
            .body(Body::from(r#"{"item":"tea","quantity":1}"#))
            .unwrap();
        assert_eq!(
            send(no_content_type).await,
            (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                json!({ "error": "Expected request with `Content-Type: application/json`" })
            )
        );
    }
}
