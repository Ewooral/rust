//! Middleware: code that wraps every request. Your own (`from_fn`) and
//! ready-made ones from `tower-http`: request logging (`TraceLayer`),
//! timeouts (`TimeoutLayer`) and CORS (`CorsLayer`).
//! Run: cargo run -p axum_basics --example middleware

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::Request;
use axum::http::{Method, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::get;
use std::time::Duration;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

// A middleware is an async fn that gets the request and `next` (the rest of
// the app). Code before `next.run` sees the request; code after sees the
// response.
async fn outer(request: Request, next: Next) -> Response {
    println!("  outer: before");
    let response = next.run(request).await;
    println!("  outer: after ({})", response.status());
    response
}

async fn inner(request: Request, next: Next) -> Response {
    println!("    inner: before");
    let response = next.run(request).await;
    println!("    inner: after");
    response
}

async fn handler() -> &'static str {
    println!("      handler runs");
    "hi"
}

async fn slow() -> &'static str {
    tokio::time::sleep(Duration::from_millis(200)).await;
    "finally"
}

async fn send(app: &Router, request: axum::http::Request<Body>) -> Response {
    app.clone().oneshot(request).await.unwrap()
}

#[tokio::main]
async fn main() {
    // 1. Order. With ServiceBuilder the FIRST layer listed is the OUTERMOST.
    //    (Calling `.layer()` on the Router one by one is the other way
    //    round: each new `.layer()` wraps everything before it.)
    println!("1. layer order");
    let app = Router::new().route("/", get(handler)).layer(
        ServiceBuilder::new()
            .layer(middleware::from_fn(outer))
            .layer(middleware::from_fn(inner)),
    );
    send(
        &app,
        axum::http::Request::get("/").body(Body::empty()).unwrap(),
    )
    .await;

    // 2. Timeouts: a request that takes longer than 50 ms is cut off.
    //    The handler's future is dropped (cancelled) at its `.await`.
    println!("\n2. TimeoutLayer (limit 50 ms, handler needs 200 ms)");
    let app = Router::new()
        .route("/slow", get(slow))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_millis(50),
        ));
    let started = std::time::Instant::now();
    let response = send(
        &app,
        axum::http::Request::get("/slow")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    println!(
        "  -> {} after about {} ms",
        response.status(),
        started.elapsed().as_millis() / 10 * 10
    );

    // 3. CORS: a browser asks first ("preflight", an OPTIONS request) whether
    //    a page from another origin may call us. The layer answers.
    println!("\n3. CorsLayer");
    let app = Router::new().route("/", get(handler)).layer(
        CorsLayer::new()
            .allow_origin(
                "http://localhost:5173"
                    .parse::<axum::http::HeaderValue>()
                    .unwrap(),
            )
            .allow_methods([Method::GET, Method::POST])
            .allow_headers([header::CONTENT_TYPE]),
    );
    let preflight = axum::http::Request::builder()
        .method(Method::OPTIONS)
        .uri("/")
        .header(header::ORIGIN, "http://localhost:5173")
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
        .body(Body::empty())
        .unwrap();
    let response = send(&app, preflight).await;
    println!("  preflight -> {}", response.status());
    for (name, value) in response.headers() {
        if name.as_str().starts_with("access-control") {
            println!("    {name}: {}", value.to_str().unwrap());
        }
    }

    // 4. TraceLayer: one log line when a request starts and one when it
    //    finishes, with the status and how long it took.
    println!("\n4. TraceLayer (log lines come from tracing-subscriber)");
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .without_time()
        .with_ansi(false)
        .init();
    let app = Router::new()
        .route("/", get(|| async { "traced" }))
        .layer(TraceLayer::new_for_http());
    let response = send(
        &app,
        axum::http::Request::get("/").body(Body::empty()).unwrap(),
    )
    .await;
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    println!("  body: {:?}", String::from_utf8_lossy(&body));
}
