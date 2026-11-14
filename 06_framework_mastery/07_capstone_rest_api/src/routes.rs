//! The route table and the middleware around it.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::{HeaderValue, Request, StatusCode};
use axum::routing::{get, post};
use tower::ServiceBuilder;
use tower_http::request_id::{
    MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer,
};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::handlers::{self, auth, health, reviews};
use crate::state::AppState;

/// Largest request body accepted (reviews are text).
pub const BODY_LIMIT: usize = 64 * 1024;

/// Numbers requests `1`, `2`, `3`… for the `x-request-id` header. (A real
/// deployment would often use UUIDs, or keep an id sent by its load balancer:
/// `SetRequestIdLayer` leaves an existing `x-request-id` alone.)
#[derive(Clone, Default)]
struct CountingIds(Arc<AtomicU64>);

impl MakeRequestId for CountingIds {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let n = self.0.fetch_add(1, Ordering::Relaxed) + 1;
        HeaderValue::from_str(&n.to_string())
            .ok()
            .map(RequestId::new)
    }
}

/// The whole API: routes → handlers, wrapped in middleware.
pub fn build_router(state: AppState) -> Router {
    let api = Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/me", get(auth::me))
        .route("/reviews", get(reviews::list).post(reviews::create))
        .route(
            "/reviews/{id}",
            get(reviews::get)
                .patch(reviews::update)
                .delete(reviews::delete),
        );

    // ServiceBuilder lists layers OUTSIDE-IN: the first one sees the request
    // first. So: give it an id, open a tracing span that records the id,
    // copy the id onto the response, and give up after the timeout.
    let timeout = state.config.request_timeout;
    let middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::x_request_id(CountingIds::default()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let id = request
                        .headers()
                        .get("x-request-id")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("-");
                    tracing::info_span!(
                        "request",
                        method = %request.method(),
                        path = %request.uri().path(),
                        request_id = %id,
                    )
                })
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            timeout,
        ));

    Router::new()
        .route("/health", get(health::health))
        .nest("/api/v1", api)
        .fallback(handlers::not_found)
        .method_not_allowed_fallback(handlers::method_not_allowed)
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .layer(middleware)
        .with_state(state)
}
