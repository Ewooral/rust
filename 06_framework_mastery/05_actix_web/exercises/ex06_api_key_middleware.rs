//! Exercise 6: an API-key middleware with `middleware::from_fn`.
//!
//! `/health` is public. Everything under `/api` must carry the header
//! `x-api-key: open-sesame`. Implement `require_api_key`:
//!
//!   - no key, or the wrong key: answer straight away with
//!     401 and `{"error": "missing or wrong API key"}`; the handler must NOT run;
//!   - the right key: call the rest of the chain (`next.call(req).await`) and
//!     add the header `x-api-version: 1` to its response.
//!
//! The two answers have different body types, so turn them into one with
//! `map_into_left_body()` (the handler's response) and
//! `req.into_response(HttpResponse).map_into_right_body()` (your refusal).
//! The return type says `EitherBody<B>`, which is exactly that.
//!
//! Test: cargo test -p actix_web_basics --features exercises --bin ex06_api_key_middleware

#![allow(unused_variables, unused_imports)]

use std::sync::atomic::{AtomicUsize, Ordering};

use actix_web::body::{EitherBody, MessageBody};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::middleware::{Next, from_fn};
use actix_web::{App, Error, HttpResponse, web};
use serde_json::json;

const API_KEY: &str = "open-sesame";

async fn require_api_key<B: MessageBody>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<EitherBody<B>>, Error> {
    todo!()
}

/// Counts how often the secret handler really ran.
#[derive(Default)]
struct Calls(AtomicUsize);

async fn secret(calls: web::Data<Calls>) -> HttpResponse {
    calls.0.fetch_add(1, Ordering::SeqCst);
    HttpResponse::Ok().json(json!({ "treasure": "gold" }))
}

fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(|| async { "ok" }))
        .service(
            web::scope("/api")
                .wrap(from_fn(require_api_key))
                .route("/treasure", web::get().to(secret)),
        );
}

#[actix_web::main]
async fn main() {
    use actix_web::test::{self, TestRequest};
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(Calls::default()))
            .configure(configure),
    )
    .await;
    let response =
        test::call_service(&app, TestRequest::get().uri("/api/treasure").to_request()).await;
    println!("GET /api/treasure without a key -> {}", response.status());
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::test::{self, TestRequest};
    use serde_json::Value;

    async fn call(path: &str, key: Option<&str>) -> (StatusCode, Option<String>, Value, usize) {
        let calls = web::Data::new(Calls::default());
        let app = test::init_service(App::new().app_data(calls.clone()).configure(configure)).await;
        let mut request = TestRequest::get().uri(path);
        if let Some(key) = key {
            request = request.insert_header(("x-api-key", key));
        }
        let response = test::call_service(&app, request.to_request()).await;
        let status = response.status();
        let version = response
            .headers()
            .get("x-api-version")
            .map(|v| v.to_str().unwrap().to_string());
        let body = test::read_body(response).await;
        let json = serde_json::from_slice(&body).unwrap_or(Value::Null);
        (status, version, json, calls.0.load(Ordering::SeqCst))
    }

    #[actix_web::test]
    async fn health_needs_no_key() {
        let (status, version, _, _) = call("/health", None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(version, None, "the middleware is only on /api");
    }

    #[actix_web::test]
    async fn no_key_is_401_and_the_handler_never_runs() {
        let (status, _, body, calls) = call("/api/treasure", None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body, json!({ "error": "missing or wrong API key" }));
        assert_eq!(calls, 0);
    }

    #[actix_web::test]
    async fn the_wrong_key_is_401() {
        let (status, _, _, calls) = call("/api/treasure", Some("please")).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(calls, 0);
    }

    #[actix_web::test]
    async fn the_right_key_gets_through_with_a_version_header() {
        let (status, version, body, calls) = call("/api/treasure", Some(API_KEY)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(version.as_deref(), Some("1"));
        assert_eq!(body, json!({ "treasure": "gold" }));
        assert_eq!(calls, 1);
    }
}
