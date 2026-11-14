//! Beyond the Book: a server must survive clients that misbehave.
//! Run: cargo run -p web_server --example misbehaving_clients
//!
//! The Book's code calls `unwrap()` on every read. Each client below would
//! crash one of its workers (or block it forever). Our server has ONE worker
//! here on purpose: if any client could break or block it, the last, normal
//! request would never be answered.

use std::io::Write;
use std::net::TcpStream;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use web_server::{Router, Server, client};

fn main() {
    let router = Router::new(Path::new(env!("CARGO_MANIFEST_DIR")), Duration::ZERO);
    let server = Server::bind("127.0.0.1:0", 1, router)
        .unwrap()
        .with_read_timeout(Duration::from_millis(300));
    let addr = server.local_addr().unwrap();
    let handle = server.shutdown_handle().unwrap();
    let running = thread::spawn(move || server.run(None));

    // 1. Connects and leaves without a word. (The Book: `.next().unwrap()` panics.)
    drop(TcpStream::connect(addr).unwrap());
    println!("1. hung up at once: no crash");

    // 2. Connects and says nothing. Without a read timeout, the only worker
    //    would wait for this client forever.
    let silent = TcpStream::connect(addr).unwrap();
    println!("2. silent client connected; the read timeout will free the worker");

    // 3. Sends nonsense: it gets a 404, not a crash.
    let reply = client::send_raw(addr, "HELLO THERE\r\n\r\n").unwrap();
    println!("3. nonsense request -> {}", client::status_line(&reply));

    // 4. Sends 100 KB of headers: only the first 8 KB are ever read.
    let mut flood = TcpStream::connect(addr).unwrap();
    let junk = format!(
        "GET / HTTP/1.1\r\n{}",
        "X-Junk: aaaaaaaaaaaaaaaa\r\n".repeat(4_000)
    );
    let _ = flood.write_all(junk.as_bytes()); // may fail if the server hangs up first
    println!(
        "4. sent {} KB of headers; the server reads at most 8 KB",
        junk.len() / 1024
    );

    // A normal visitor still gets the page.
    let start = Instant::now();
    let reply = client::get(addr, "/").unwrap();
    println!(
        "\nnormal visitor -> {} after {:.1?}",
        client::status_line(&reply),
        start.elapsed()
    );

    drop(silent);
    drop(flood);
    handle.shutdown();
    running.join().unwrap();
}
