//! Graceful shutdown: stop accepting new connections, but let the requests
//! already in progress finish.
//! Run: cargo run -p axum_basics --example graceful_shutdown
//!
//! A real server waits for Ctrl+C (`tokio::signal::ctrl_c()`), as
//! `src/main.rs` does. Here a oneshot channel plays the part of Ctrl+C, so
//! the example can stop itself: it starts a slow request, "presses Ctrl+C"
//! while that request is still running, and shows that the slow request
//! still completes while a new connection is no longer accepted.

use axum::Router;
use axum::routing::get;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio::time::{sleep, timeout};

#[tokio::main]
async fn main() {
    let start = Instant::now();
    let ms = move || start.elapsed().as_millis() / 50 * 50; // rounded, for tidy output

    let app = Router::new().route("/", get(|| async { "quick" })).route(
        "/slow",
        get(|| async {
            sleep(Duration::from_millis(400)).await;
            "slow, but finished"
        }),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (ctrl_c_tx, ctrl_c_rx) = oneshot::channel::<()>();

    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                ctrl_c_rx.await.ok(); // "wait for Ctrl+C"
                println!(
                    "[{:>3} ms] server: shutdown signal, no new connections",
                    ms()
                );
            })
            .await
    });

    let quick = axum_basics::client::get(addr, "/").await.unwrap();
    println!(
        "[{:>3} ms] client: GET / -> {} {:?}",
        ms(),
        quick.status,
        quick.body
    );

    let slow = tokio::spawn(async move { axum_basics::client::get(addr, "/slow").await });
    sleep(Duration::from_millis(100)).await; // let /slow get going

    println!(
        "[{:>3} ms] main: \"Ctrl+C\" while /slow is still running",
        ms()
    );
    ctrl_c_tx.send(()).unwrap();
    sleep(Duration::from_millis(50)).await;

    // Linux refuses at once; Windows retries a refused connect for about two
    // seconds, so give up after 100 ms either way.
    match timeout(Duration::from_millis(100), TcpStream::connect(addr)).await {
        Ok(Ok(_)) => println!("[{:>3} ms] client: new connection accepted?!", ms()),
        Ok(Err(e)) => println!(
            "[{:>3} ms] client: new connection refused ({:?})",
            ms(),
            e.kind()
        ),
        Err(_) => println!(
            "[{:>3} ms] client: new connection not accepted (no answer in 100 ms)",
            ms()
        ),
    }

    let slow = slow.await.unwrap().unwrap();
    println!(
        "[{:>3} ms] client: GET /slow -> {} {:?}",
        ms(),
        slow.status,
        slow.body
    );
    server.await.unwrap().unwrap();
    println!(
        "[{:>3} ms] main: serve() returned: every connection is closed",
        ms()
    );
}
