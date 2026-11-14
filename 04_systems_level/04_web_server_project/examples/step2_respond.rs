//! Book 21.1, steps 3-5: write a response, return real HTML, and choose the
//! page from the request line (200 or 404).
//! Run: cargo run -p web_server --example step2_respond
//!
//! This is the Book's single-threaded `handle_connection`, nearly line for
//! line. It serves two connections from a pretend browser, then exits.

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread;

use web_server::client;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    let browser = thread::spawn(move || {
        for path in ["/", "/something-else"] {
            let response = client::get(addr, path).unwrap();
            let (head, body) = response.split_once("\r\n\r\n").unwrap();
            println!("\nbrowser asked for {path} and got:");
            for line in head.lines() {
                println!("  {line}");
            }
            let heading = body.lines().find(|l| l.contains("<h1>")).unwrap_or("");
            println!(
                "  (blank line, then {} bytes of HTML: {})",
                body.len(),
                heading.trim()
            );
        }
    });

    for stream in listener.incoming().take(2) {
        handle_connection(stream.unwrap());
    }
    browser.join().unwrap();
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    // Only the first line matters for routing: "GET / HTTP/1.1".
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    // The Book's refactor: decide only what differs, then share the rest.
    let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    // The pages sit next to Cargo.toml; the Book reads them from the
    // current directory, which only works when you run from this folder.
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(filename);
    let contents = fs::read_to_string(path).unwrap();
    let length = contents.len();

    // status line CRLF headers CRLF CRLF body
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}
