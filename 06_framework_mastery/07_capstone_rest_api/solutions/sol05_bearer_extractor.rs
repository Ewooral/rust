//! Solution to exercise 5: an authentication extractor (`FromRequestParts`).
//!
//! Handlers that take a `CurrentUser` argument must only run for requests
//! with a valid `Authorization: Bearer <token>` header. Implement
//! `FromRequestParts<AppState> for CurrentUser`:
//!
//!   1. Read the `Authorization` header from `parts.headers`.
//!      Missing                        -> AuthError("missing token")
//!   2. It must be `Bearer <token>` (exactly that word, one space, then a
//!      non-empty token).              -> else AuthError("malformed Authorization header")
//!   3. Look the token up with `state.sessions.get(token)` (a stand-in for
//!      checking a JWT signature).     -> unknown: AuthError("invalid token")
//!   4. Return `CurrentUser { name }`.
//!
//! `AuthError` already turns into a 401 with a JSON body and a
//! `www-authenticate: Bearer` header; it's the extractor's `Rejection`.
//!
//! Test: cargo test -p capstone_rest_api --features solutions --bin sol05_bearer_extractor

use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;
use axum::extract::FromRequestParts;
use axum::http::header::{AUTHORIZATION, WWW_AUTHENTICATE};
use axum::http::request::Parts;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde_json::json;

#[derive(Clone)]
struct AppState {
    /// token -> user name
    sessions: Arc<HashMap<String, String>>,
}

#[derive(Debug)]
struct CurrentUser {
    name: String,
}

#[derive(Debug)]
struct AuthError(&'static str);

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let body = axum::Json(json!({ "error": self.0 }));
        let mut response = (StatusCode::UNAUTHORIZED, body).into_response();
        response
            .headers_mut()
            .insert(WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        response
    }
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or(AuthError("missing token"))?;
        let token = header
            .to_str()
            .ok()
            .and_then(|value| value.strip_prefix("Bearer "))
            .filter(|token| !token.is_empty())
            .ok_or(AuthError("malformed Authorization header"))?;
        let name = state
            .sessions
            .get(token)
            .ok_or(AuthError("invalid token"))?;
        Ok(CurrentUser { name: name.clone() })
    }
}

async fn whoami(user: CurrentUser) -> String {
    format!("hello, {}", user.name)
}

async fn public() -> &'static str {
    "anyone can see this"
}

fn app() -> Router {
    let sessions = HashMap::from([("abc123".to_string(), "Ama".to_string())]);
    Router::new()
        .route("/whoami", get(whoami))
        .route("/public", get(public))
        .with_state(AppState {
            sessions: Arc::new(sessions),
        })
}

#[tokio::main]
async fn main() {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;
    let request = Request::get("/whoami")
        .header(AUTHORIZATION, "Bearer abc123")
        .body(Body::empty())
        .unwrap();
    let response = app().oneshot(request).await.unwrap();
    println!("GET /whoami -> {}", response.status());
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use serde_json::Value;
    use tower::ServiceExt;

    async fn get(path: &str, authorization: Option<&str>) -> (StatusCode, String, Option<String>) {
        let mut request = Request::get(path);
        if let Some(value) = authorization {
            request = request.header(AUTHORIZATION, value);
        }
        let response = app()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let challenge = response
            .headers()
            .get(WWW_AUTHENTICATE)
            .map(|v| v.to_str().unwrap().to_string());
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        (
            status,
            String::from_utf8_lossy(&body).into_owned(),
            challenge,
        )
    }

    fn error_of(body: &str) -> String {
        let json: Value = serde_json::from_str(body).unwrap();
        json["error"].as_str().unwrap().to_string()
    }

    #[tokio::test]
    async fn a_valid_token_reaches_the_handler() {
        let (status, body, _) = get("/whoami", Some("Bearer abc123")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "hello, Ama");
    }

    #[tokio::test]
    async fn no_header_is_401_with_a_challenge() {
        let (status, body, challenge) = get("/whoami", None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(error_of(&body), "missing token");
        assert_eq!(challenge.as_deref(), Some("Bearer"));
    }

    #[tokio::test]
    async fn malformed_and_unknown_tokens_are_401() {
        for header in ["Basic abc123", "Bearer", "Bearer ", "abc123"] {
            let (status, body, _) = get("/whoami", Some(header)).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED, "{header:?}");
            assert_eq!(
                error_of(&body),
                "malformed Authorization header",
                "{header:?}"
            );
        }
        let (status, body, _) = get("/whoami", Some("Bearer nope")).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(error_of(&body), "invalid token");
    }

    #[tokio::test]
    async fn routes_without_the_extractor_stay_public() {
        let (status, body, _) = get("/public", None).await;
        assert_eq!(
            (status, body.as_str()),
            (StatusCode::OK, "anyone can see this")
        );
    }
}
