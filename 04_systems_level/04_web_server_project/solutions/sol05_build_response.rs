//! Solution to exercise 5: build an HTTP response.
//!
//! Implement:
//!   - `reason_phrase(status)`: 200 -> "OK", 400 -> "Bad Request",
//!     404 -> "Not Found", 500 -> "Internal Server Error", anything else
//!     -> "Unknown".
//!   - `Response::to_http(&self)`: the exact text to send.
//!
//! For the body "hi" that text is the following, where each `\r\n` is a
//! real carriage return + line feed and the body has no line ending:
//!
//! ```text
//! HTTP/1.1 200 OK\r\n
//! Content-Length: 2\r\n
//! Content-Type: text/html\r\n
//! \r\n
//! hi
//! ```
//!
//! Careful: `Content-Length` is the body's length in BYTES. "café" is 4
//! characters but 5 bytes, and a browser trusts the number you send.
//!
//! Test: cargo test -p web_server --features solutions --bin sol05_build_response

struct Response {
    status: u16,
    content_type: &'static str,
    body: String,
}

impl Response {
    fn html(status: u16, body: &str) -> Response {
        Response {
            status,
            content_type: "text/html",
            body: body.to_string(),
        }
    }

    fn to_http(&self) -> String {
        format!(
            "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n{}",
            self.status,
            reason_phrase(self.status),
            self.body.len(), // String::len is the length in BYTES
            self.content_type,
            self.body
        )
    }
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    }
}

fn main() {
    let response = Response::html(404, "<h1>Oops!</h1>");
    print!("{}", response.to_http().replace("\r\n", "\\r\\n\n"));
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reason_phrases() {
        assert_eq!(reason_phrase(200), "OK");
        assert_eq!(reason_phrase(400), "Bad Request");
        assert_eq!(reason_phrase(404), "Not Found");
        assert_eq!(reason_phrase(500), "Internal Server Error");
        assert_eq!(reason_phrase(299), "Unknown");
    }

    #[test]
    fn exact_response_text() {
        assert_eq!(
            Response::html(200, "hi").to_http(),
            "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Type: text/html\r\n\r\nhi"
        );
        let missing = Response::html(404, "").to_http();
        assert!(missing.starts_with("HTTP/1.1 404 Not Found\r\n"));
        assert!(missing.ends_with("Content-Length: 0\r\nContent-Type: text/html\r\n\r\n"));
    }

    #[test]
    fn content_length_counts_bytes_not_characters() {
        let body = "naïve café ☕"; // 12 characters, 16 bytes
        let text = Response::html(200, body).to_http();
        assert!(text.contains("Content-Length: 16\r\n"), "{text}");
        let (_, sent_body) = text.split_once("\r\n\r\n").unwrap();
        assert_eq!(sent_body, body);
    }
}
