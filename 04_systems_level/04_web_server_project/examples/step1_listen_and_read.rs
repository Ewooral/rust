//! Book 21.1, steps 1-2: listen for TCP connections and read an HTTP request.
//! Run: cargo run -p web_server --example step1_listen_and_read
//!
//! The Book binds 127.0.0.1:7878 and loops forever; open the address in a
//! browser to see a request. This example plays the browser itself from a
//! second thread, handles exactly ONE connection, and exits.

use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::thread;

use web_server::client;

fn main() {
    // Port 0: the operating system picks a free port, so this never clashes
    // with a server already using 7878.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    println!("Listening on {addr}");

    // The "browser": connects and sends a request like a real one would.
    let browser = thread::spawn(move || {
        client::send_raw(
            addr,
            &format!(
                "GET / HTTP/1.1\r\nHost: {addr}\r\nUser-Agent: example-browser\r\nAccept: text/html\r\n\r\n"
            ),
        )
    });

    // `incoming()` yields one TcpStream per connection, forever. `.take(1)`
    // stops after the first so the example ends.
    for stream in listener.incoming().take(1) {
        let stream = stream.unwrap();
        println!("Connection established!");

        // An HTTP request is lines of text that end with a blank line.
        let buf_reader = BufReader::new(&stream);
        let http_request: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();
        println!("Request: {http_request:#?}");
    } // the stream is dropped here, which closes the connection

    let reply = browser.join().unwrap().unwrap();
    println!(
        "The browser got {} bytes back: we haven't written a response yet (step 2).",
        reply.len()
    );
}
