//! End-to-end tests: start the real server on a free port, talk to it over
//! TCP like a browser would, then shut it down.
//!
//! Every wait has a timeout, so a broken server fails the test instead of
//! hanging it.
//!
//! Run: cargo test -p web_server --test server

use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use web_server::{Router, Server, ShutdownHandle, client};

const PATIENCE: Duration = Duration::from_secs(10);

/// Starts a server on 127.0.0.1:0 (the OS picks a free port) in a thread.
/// Returns its address, a handle to stop it, and a channel that receives
/// `()` when `run` has returned (all workers joined).
fn start(
    threads: usize,
    slow: Duration,
    limit: Option<usize>,
) -> (SocketAddr, ShutdownHandle, mpsc::Receiver<()>) {
    let router = Router::new(Path::new(env!("CARGO_MANIFEST_DIR")), slow);
    let server = Server::bind("127.0.0.1:0", threads, router)
        .expect("bind to a free port")
        .with_read_timeout(Duration::from_secs(1));
    let addr = server.local_addr().unwrap();
    let handle = server.shutdown_handle().unwrap();
    let (done_tx, done_rx) = mpsc::channel();
    thread::spawn(move || {
        server.run(limit);
        let _ = done_tx.send(());
    });
    (addr, handle, done_rx)
}

fn send(addr: SocketAddr, request: &str) -> String {
    client::send_raw(addr, request).expect("a response")
}

fn get(addr: SocketAddr, path: &str) -> String {
    client::get(addr, path).expect("a response")
}

#[test]
fn serves_pages_and_404s_then_shuts_down() {
    let (addr, handle, done) = start(2, Duration::ZERO, None);

    let home = get(addr, "/");
    assert!(home.starts_with("HTTP/1.1 200 OK\r\n"), "{home}");
    assert!(home.contains("Hi from Rust"));

    let missing = get(addr, "/does-not-exist");
    assert!(missing.starts_with("HTTP/1.1 404 NOT FOUND\r\n"));
    assert!(missing.contains("Oops!"));

    let garbage = send(addr, "HELLO?\r\n\r\n");
    assert!(garbage.starts_with("HTTP/1.1 404"));

    handle.shutdown();
    done.recv_timeout(PATIENCE).expect("server stopped");
}

#[test]
fn a_slow_request_does_not_block_a_fast_one() {
    let (addr, handle, done) = start(2, Duration::from_millis(600), None);

    let (tx, rx) = mpsc::channel();
    let slow_tx = tx.clone();
    thread::spawn(move || {
        let response = get(addr, "/sleep");
        slow_tx.send(("slow", response, Instant::now())).unwrap();
    });
    thread::sleep(Duration::from_millis(100)); // let /sleep reach a worker first
    let fast = get(addr, "/");
    tx.send(("fast", fast, Instant::now())).unwrap();

    // With two workers, "/" is served while the other worker sleeps, so it
    // finishes first. With one worker it would wait behind /sleep.
    let first = rx.recv_timeout(PATIENCE).unwrap();
    let second = rx.recv_timeout(PATIENCE).unwrap();
    assert_eq!((first.0, second.0), ("fast", "slow"));
    assert!(first.1.starts_with("HTTP/1.1 200 OK") && second.1.starts_with("HTTP/1.1 200 OK"));

    handle.shutdown();
    done.recv_timeout(PATIENCE).expect("server stopped");
}

#[test]
fn a_limit_stops_the_server_like_the_books_take_2() {
    let (addr, _handle, done) = start(2, Duration::ZERO, Some(2));
    assert!(get(addr, "/").contains("200 OK"));
    assert!(get(addr, "/").contains("200 OK"));
    done.recv_timeout(PATIENCE)
        .expect("stopped by itself after 2 connections");
}

#[test]
fn a_silent_client_times_out_without_blocking_others() {
    let (addr, handle, done) = start(1, Duration::ZERO, None);
    // Connect and say nothing. The ONLY worker is stuck reading until the
    // 1-second read timeout frees it.
    let _silent = TcpStream::connect_timeout(&addr, PATIENCE).unwrap();
    thread::sleep(Duration::from_millis(100));
    let started = Instant::now();
    let response = get(addr, "/");
    assert!(response.starts_with("HTTP/1.1 200 OK"));
    assert!(
        started.elapsed() < PATIENCE,
        "served once the timeout freed the worker"
    );

    handle.shutdown();
    done.recv_timeout(PATIENCE).expect("server stopped");
}
