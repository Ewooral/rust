//! Book 21.2, "Simulating a Slow Request": one slow request makes every
//! other request wait, because a single thread handles them one at a time.
//! Run: cargo run -p web_server --example step3_slow_single_thread
//!
//! Two pretend browsers: A asks for /sleep (1 second here, 5 in the Book),
//! and B asks for / a moment later. Watch how long B waits.

use std::net::TcpListener;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use web_server::{Router, client, handle_connection};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let router = Router::new(
        Path::new(env!("CARGO_MANIFEST_DIR")),
        Duration::from_secs(1),
    );

    let start = Instant::now();
    let a = thread::spawn(move || {
        client::get(addr, "/sleep").unwrap();
        println!("A (/sleep) done after {:.1?}", start.elapsed());
    });
    thread::sleep(Duration::from_millis(100)); // make sure A connects first
    let b = thread::spawn(move || {
        client::get(addr, "/").unwrap();
        println!(
            "B (/)      done after {:.1?}  <- had to wait for A",
            start.elapsed()
        );
    });

    // ONE thread handles every connection, in order.
    for stream in listener.incoming().take(2) {
        handle_connection(stream.unwrap(), &router).unwrap();
    }
    a.join().unwrap();
    b.join().unwrap();
}
