//! Solution to exercise 5: every error as JSON, including the extractors' own errors.
//!
//! Two parts:
//!
//! 1. Implement `ResponseError for ApiError`:
//!      - `status_code`: NotFound -> 404, Invalid -> 422, BadRequest -> 400
//!      - `error_response`: `{"error": "<Display text>"}`, and for `Invalid`
//!        also `"details": [ ...the problems... ]`
//!
//! 2. Implement `json_config()`: a `web::JsonConfig` whose `error_handler`
//!    turns Actix's `JsonPayloadError` into `ApiError::BadRequest(err.to_string())`
//!    (return `ApiError::BadRequest(...).into()`: anything that implements
//!    `ResponseError` converts into an `actix_web::Error`). Without it, a
//!    malformed body gets Actix's plain-text 400.
//!
//! Test: cargo test -p actix_web_basics --features solutions --bin sol05_json_errors

use std::fmt;

use actix_web::http::StatusCode;
use actix_web::{App, HttpResponse, ResponseError, post, web};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, PartialEq, Eq)]
enum ApiError {
    NotFound(String),
    Invalid(Vec<String>),
    BadRequest(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NotFound(what) => write!(f, "{what} not found"),
            ApiError::Invalid(_) => write!(f, "validation failed"),
            ApiError::BadRequest(why) => write!(f, "bad request: {why}"),
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Invalid(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let body = match self {
            ApiError::Invalid(details) => json!({ "error": self.to_string(), "details": details }),
            _ => json!({ "error": self.to_string() }),
        };
        HttpResponse::build(self.status_code()).json(body)
    }
}

fn json_config() -> web::JsonConfig {
    web::JsonConfig::default()
        .error_handler(|err, _req| ApiError::BadRequest(err.to_string()).into())
}

#[derive(Debug, Deserialize)]
struct Order {
    item: String,
    quantity: u32,
}

#[post("/orders")]
async fn order(body: web::Json<Order>) -> Result<HttpResponse, ApiError> {
    let mut problems = Vec::new();
    if body.item.trim().is_empty() {
        problems.push("item must not be empty".to_string());
    }
    if body.quantity == 0 || body.quantity > 10 {
        problems.push("quantity must be 1 to 10".to_string());
    }
    if !problems.is_empty() {
        return Err(ApiError::Invalid(problems));
    }
    if body.item != "tea" {
        return Err(ApiError::NotFound(body.item.clone()));
    }
    Ok(HttpResponse::Created().json(json!({ "ordered": body.quantity })))
}

#[actix_web::main]
async fn main() {
    use actix_web::test::{self, TestRequest};
    let app = test::init_service(App::new().app_data(json_config()).service(order)).await;
    let request = TestRequest::post()
        .uri("/orders")
        .set_json(json!({ "item": "cake", "quantity": 1 }));
    let response = test::call_service(&app, request.to_request()).await;
    println!("POST /orders -> {}", response.status());
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::{self, TestRequest};
    use serde_json::Value;

    async fn post(body: TestRequest) -> (StatusCode, Value) {
        let app = test::init_service(App::new().app_data(json_config()).service(order)).await;
        let response = test::call_service(&app, body.uri("/orders").to_request()).await;
        let status = response.status();
        let body = test::read_body(response).await;
        (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
    }

    #[actix_web::test]
    async fn a_good_order_is_created() {
        let (status, body) =
            post(TestRequest::post().set_json(json!({ "item": "tea", "quantity": 2 }))).await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body, json!({ "ordered": 2 }));
    }

    #[actix_web::test]
    async fn not_found_is_404_json() {
        let (status, body) =
            post(TestRequest::post().set_json(json!({ "item": "cake", "quantity": 1 }))).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body, json!({ "error": "cake not found" }));
    }

    #[actix_web::test]
    async fn invalid_is_422_with_details() {
        let (status, body) =
            post(TestRequest::post().set_json(json!({ "item": " ", "quantity": 0 }))).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            json!({
                "error": "validation failed",
                "details": ["item must not be empty", "quantity must be 1 to 10"]
            })
        );
    }

    #[actix_web::test]
    async fn malformed_json_is_400_json() {
        let (status, body) = post(
            TestRequest::post()
                .insert_header(("content-type", "application/json"))
                .set_payload("{not json"),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        let message = body["error"].as_str().unwrap();
        assert!(
            message.starts_with("bad request: Json deserialize error"),
            "{message}"
        );
    }
}
