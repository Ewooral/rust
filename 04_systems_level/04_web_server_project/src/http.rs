//! Requests in, responses out (Book 21.1).
//!
//! An HTTP request is plain text: a request line, header lines, a blank
//! line, then an optional body. Lines end with CRLF (`\r\n`):
//!
//! ```text
//! GET / HTTP/1.1
//! Host: 127.0.0.1:7878
//! User-Agent: curl/8.5.0
//!
//! ```
//!
//! A response has the same shape: a status line, headers, a blank line and
//! the body.
//!
//! `handle_connection` is generic over `Read + Write`, so the same code
//! serves a real `TcpStream` and an in-memory fake in the unit tests.

use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

/// The most we read of a request's head. A client sending endless headers
/// can't make the server use endless memory.
pub const MAX_HEAD_BYTES: u64 = 8 * 1024;

/// A response ready to send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub reason: &'static str,
    pub body: String,
}

impl Response {
    /// `HTTP/1.1 200 OK`
    pub fn status_line(&self) -> String {
        format!("HTTP/1.1 {} {}", self.status, self.reason)
    }

    /// The exact bytes to write to the client.
    ///
    /// `Content-Length` counts BYTES, not characters, which is why it's
    /// `self.body.len()`. `Connection: close` tells the client we hang up
    /// after one response, so it doesn't wait for more.
    pub fn to_bytes(&self) -> Vec<u8> {
        format!(
            "{}\r\nContent-Length: {}\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n{}",
            self.status_line(),
            self.body.len(),
            self.body
        )
        .into_bytes()
    }
}

/// Decides what to send back for each request line.
#[derive(Debug, Clone)]
pub struct Router {
    /// The folder holding `hello.html` and `404.html`.
    pages: PathBuf,
    /// How long `/sleep` sleeps: 5 seconds in the Book, shorter in tests.
    slow: Duration,
}

impl Router {
    pub fn new(pages: impl Into<PathBuf>, slow: Duration) -> Router {
        Router {
            pages: pages.into(),
            slow,
        }
    }

    /// The Book's routing: match on the whole request line.
    pub fn respond(&self, request_line: &str) -> Response {
        let (status, reason, page) = match request_line {
            "GET / HTTP/1.1" => (200, "OK", "hello.html"),
            "GET /sleep HTTP/1.1" => {
                thread::sleep(self.slow); // simulate a slow request
                (200, "OK", "hello.html")
            }
            _ => (404, "NOT FOUND", "404.html"),
        };

        // The Book unwraps here. A missing page is the server's fault, not
        // the client's, so we answer 500 instead of crashing the worker.
        match fs::read_to_string(self.pages.join(page)) {
            Ok(body) => Response {
                status,
                reason,
                body,
            },
            Err(e) => Response {
                status: 500,
                reason: "INTERNAL SERVER ERROR",
                body: format!("<h1>500</h1><p>could not read {page}: {e}</p>"),
            },
        }
    }
}

/// Reads the request line and headers, up to the blank line that ends them.
///
/// Reading the WHOLE head (not just the first line, as the Book's final
/// version does) matters: if the server closes the socket while unread
/// bytes are still waiting, the operating system may reset the connection
/// and the client can lose the response.
pub fn read_request_head(reader: impl BufRead) -> io::Result<Vec<String>> {
    let mut head = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            break;
        }
        head.push(line);
    }
    Ok(head)
}

/// Serves one connection: read the request, write the response.
///
/// Generic over `Read + Write` instead of taking a `TcpStream`, so tests can
/// pass an in-memory stream. Returns errors instead of panicking: one bad
/// client must not take down a worker.
pub fn handle_connection<S: Read + Write>(mut stream: S, router: &Router) -> io::Result<()> {
    // `&mut stream` LENDS the stream to `take` (instead of giving it away),
    // so we can still write to it afterwards.
    let limited = (&mut stream).take(MAX_HEAD_BYTES);
    let head = read_request_head(BufReader::new(limited))?;

    let Some(request_line) = head.first() else {
        return Ok(()); // connected, sent nothing, left: nothing to answer
    };

    let response = router.respond(request_line);
    stream.write_all(&response.to_bytes())?;
    stream.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::path::Path;

    fn router() -> Router {
        Router::new(Path::new(env!("CARGO_MANIFEST_DIR")), Duration::ZERO)
    }

    /// An in-memory "socket": reads come from `input`, writes go to `output`.
    struct FakeStream {
        input: Cursor<Vec<u8>>,
        output: Vec<u8>,
    }

    impl FakeStream {
        fn new(request: &str) -> FakeStream {
            FakeStream {
                input: Cursor::new(request.as_bytes().to_vec()),
                output: Vec::new(),
            }
        }
    }

    impl Read for FakeStream {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.input.read(buf)
        }
    }

    impl Write for FakeStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.output.write(buf)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn routes_like_the_book() {
        let r = router();
        let home = r.respond("GET / HTTP/1.1");
        assert_eq!(home.status_line(), "HTTP/1.1 200 OK");
        assert!(home.body.contains("Hi from Rust"));
        assert_eq!(r.respond("GET /sleep HTTP/1.1").status, 200);
        let missing = r.respond("GET /nope HTTP/1.1");
        assert_eq!(missing.status_line(), "HTTP/1.1 404 NOT FOUND");
        assert!(missing.body.contains("Oops!"));
        assert_eq!(r.respond("POST / HTTP/1.1").status, 404);
    }

    #[test]
    fn missing_page_is_a_500_not_a_panic() {
        let r = Router::new("no/such/folder", Duration::ZERO);
        assert_eq!(r.respond("GET / HTTP/1.1").status, 500);
    }

    #[test]
    fn response_bytes_have_headers_and_byte_length() {
        let response = Response {
            status: 200,
            reason: "OK",
            body: String::from("café"), // 4 characters, 5 bytes
        };
        let text = String::from_utf8(response.to_bytes()).unwrap();
        assert!(text.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(text.contains("Content-Length: 5\r\n"));
        assert!(text.ends_with("\r\n\r\ncafé"));
    }

    #[test]
    fn reads_the_whole_head_and_stops_at_the_blank_line() {
        let raw = "GET / HTTP/1.1\r\nHost: x\r\nAccept: */*\r\n\r\nbody we ignore";
        let head = read_request_head(Cursor::new(raw)).unwrap();
        assert_eq!(head, ["GET / HTTP/1.1", "Host: x", "Accept: */*"]);
    }

    #[test]
    fn handle_connection_writes_a_full_response() {
        let mut stream = FakeStream::new("GET / HTTP/1.1\r\nHost: test\r\n\r\n");
        handle_connection(&mut stream, &router()).unwrap();
        let reply = String::from_utf8(stream.output).unwrap();
        assert!(reply.starts_with("HTTP/1.1 200 OK\r\n"));
        let (head, body) = reply.split_once("\r\n\r\n").unwrap();
        assert!(head.contains(&format!("Content-Length: {}", body.len())));
    }

    #[test]
    fn handle_connection_ignores_a_silent_client() {
        let mut stream = FakeStream::new("");
        handle_connection(&mut stream, &router()).unwrap();
        assert!(stream.output.is_empty());
    }

    #[test]
    fn an_endless_head_is_cut_off() {
        let flood = format!(
            "GET / HTTP/1.1\r\n{}",
            "X-Junk: aaaaaaaa\r\n".repeat(10_000)
        );
        let limited = Cursor::new(flood).take(MAX_HEAD_BYTES);
        let head = read_request_head(BufReader::new(limited)).unwrap();
        assert!(head.len() < 10_001, "the head was cut short");
        let kept: usize = head.iter().map(String::len).sum();
        assert!(kept <= MAX_HEAD_BYTES as usize);
    }
}
