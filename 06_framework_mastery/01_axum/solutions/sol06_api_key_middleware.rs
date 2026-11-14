//! Solution to exercise 6: write a middleware that guards some routes with an API key.
//!
//! `app` is finished: the `/admin` routes are wrapped with
//! `route_layer(middleware::from_fn_with_state(config, require_api_key))`,
//! and the public routes are not. Implement `require_api_key`:
//!
//! - no `x-api-key` header            -> 401, plain-text body `missing x-api-key header`
//! - a header that isn't `config.api_key` -> 403, body `wrong api key`
//! - the right key -> let the request through (`next.run(request).await`)
//!   and add the header `x-auth: ok` to the response on its way back.
//!
//! The middleware never reaches the handler when it refuses: it returns
//! its own response instead of calling `next`.
//!
//! Hints: `request.headers().get("x-api-key")` gives `Option<&HeaderValue>`;
//! compare with `value == config.api_key.as_str()`; build a refusal with
//! `(StatusCode::UNAUTHORIZED, "...").into_response()`; on the way back,
//! `response.headers_mut().insert("x-auth", HeaderValue::from_static("ok"))`.
//!
//! Test: cargo test -p axum_basics --features solutions --bin sol06_api_key_middleware

use axum::Router;
use axum::extract::{Request, State};
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get};

#[derive(Clone)]
struct Config {
    api_key: String,
}

async fn require_api_key(State(config): State<Config>, request: Request, next: Next) -> Response {
    match request.headers().get("x-api-key") {
        None => (StatusCode::UNAUTHORIZED, "missing x-api-key header").into_response(),
        Some(key) if key != config.api_key.as_str() => {
            (StatusCode::FORBIDDEN, "wrong api key").into_response()
        }
        Some(_) => {
            let mut response = next.run(request).await; // the handler runs here
            response
                .headers_mut()
                .insert("x-auth", HeaderValue::from_static("ok"));
            response
        }
    }
}

// ----- finished: no changes needed below this line -------------------------

fn app(config: Config) -> Router {
    let admin = Router::new()
        .route("/stats", get(|| async { "42 users" }))
        .route("/users/{id}", delete(|| async { "deleted" }))
        // `route_layer` runs the middleware only for routes that matched, so
        // an unknown /admin path is still a plain 404, not a 401.
        .route_layer(middleware::from_fn_with_state(config, require_api_key));

    Router::new()
        .route("/", get(|| async { "public page" }))
        .nest("/admin", admin)
}

#[tokio::main]
async fn main() {
    use axum::body::{Body, to_bytes};
    use tower::ServiceExt;

    let app = app(Config {
        api_key: "s3cret".to_string(),
    });
    for (uri, key) in [
        ("/", None),
        ("/admin/stats", None),
        ("/admin/stats", Some("s3cret")),
    ] {
        let mut request = axum::http::Request::get(uri);
        if let Some(key) = key {
            request = request.header("x-api-key", key);
        }
        let response = app
            .clone()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        println!(
            "GET {uri} (key {key:?}) -> {status} {}",
            String::from_utf8_lossy(&body)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use tower::ServiceExt;

    async fn call(
        method: &str,
        uri: &str,
        key: Option<&str>,
    ) -> (StatusCode, Option<String>, String) {
        let app = app(Config {
            api_key: "s3cret".to_string(),
        });
        let mut request = axum::http::Request::builder().method(method).uri(uri);
        if let Some(key) = key {
            request = request.header("x-api-key", key);
        }
        let response = app
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let auth = response
            .headers()
            .get("x-auth")
            .map(|v| v.to_str().unwrap().to_string());
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, auth, String::from_utf8(body.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn public_routes_need_no_key() {
        assert_eq!(
            call("GET", "/", None).await,
            (StatusCode::OK, None, "public page".to_string())
        );
    }

    #[tokio::test]
    async fn admin_routes_refuse_missing_or_wrong_keys() {
        assert_eq!(
            call("GET", "/admin/stats", None).await,
            (
                StatusCode::UNAUTHORIZED,
                None,
                "missing x-api-key header".to_string()
            )
        );
        assert_eq!(
            call("DELETE", "/admin/users/7", Some("guess")).await,
            (StatusCode::FORBIDDEN, None, "wrong api key".to_string())
        );
        // An unknown admin path is still a 404: route_layer didn't run.
        assert_eq!(
            call("GET", "/admin/nope", None).await.0,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn the_right_key_gets_through_and_is_stamped() {
        assert_eq!(
            call("GET", "/admin/stats", Some("s3cret")).await,
            (
                StatusCode::OK,
                Some("ok".to_string()),
                "42 users".to_string()
            )
        );
        assert_eq!(
            call("DELETE", "/admin/users/7", Some("s3cret")).await.2,
            "deleted"
        );
    }
}
