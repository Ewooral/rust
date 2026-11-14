//! Book 21.3: graceful shutdown and cleanup.
//! Run: cargo run -p web_server --example step6_graceful_shutdown
//!
//! Part 1 is the Book's ending: serve two connections (`.take(2)`), then
//! let the pool's `Drop` finish the work in progress and join every worker.
//! Part 2 (beyond the Book) stops a server from another thread with a
//! `ShutdownHandle`, even while a slow request is still running.

use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use web_server::{Router, Server, client};

fn router() -> Router {
    Router::new(
        Path::new(env!("CARGO_MANIFEST_DIR")),
        Duration::from_millis(500),
    )
}

fn main() {
    println!("== part 1: the Book's take(2) ==");
    let server = Server::bind("127.0.0.1:0", 4, router()).unwrap();
    let addr = server.local_addr().unwrap();
    let browser = thread::spawn(move || {
        for path in ["/", "/sleep"] {
            let response = client::get(addr, path).unwrap();
            println!("browser: {path} -> {}", client::status_line(&response));
        }
    });
    server.run(Some(2)); // returns after accepting 2 connections, then drops the pool
    browser.join().unwrap();

    println!("\n== part 2: stopping from another thread ==");
    let server = Server::bind("127.0.0.1:0", 2, router()).unwrap();
    let addr = server.local_addr().unwrap();
    let handle = server.shutdown_handle().unwrap();
    let running = thread::spawn(move || server.run(None)); // no limit: runs until told

    let slow = thread::spawn(move || {
        let start = Instant::now();
        let response = client::get(addr, "/sleep").unwrap();
        println!(
            "slow browser: {} after {:.1?}, even though shutdown began meanwhile",
            client::status_line(&response),
            start.elapsed()
        );
    });
    thread::sleep(Duration::from_millis(100)); // let /sleep reach a worker
    println!("asking the server to stop");
    handle.shutdown(); // sets a flag, then connects to wake up accept()
    running.join().unwrap(); // returns once every worker has finished
    slow.join().unwrap();
    println!("server stopped cleanly");
}
