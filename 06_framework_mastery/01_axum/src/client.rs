//! A tiny HTTP/1.1 client written on a raw `tokio::net::TcpStream`, for the
//! examples and the end-to-end test. HTTP is still just text over TCP, as
//! in lesson 04/04: write a request, read the reply until the server hangs
//! up (`Connection: close`), split the head from the body.
//!
//! Real code would use a client crate such as `reqwest` or `hyper-util`;
//! this one keeps the lesson free of extra dependencies. It only handles
//! bodies with a `Content-Length` (every response this lesson sends has
//! one), and every call has a 5-second limit so a test can never hang.

use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Response {
    /// The first header with this name (names are case-insensitive).
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

/// Sends one request and returns the parsed response. `body`, if any, is
/// sent as JSON.
pub async fn send(
    addr: SocketAddr,
    method: &str,
    path: &str,
    body: Option<&str>,
) -> io::Result<Response> {
    let exchange = async {
        let mut stream = TcpStream::connect(addr).await?;
        let mut request =
            format!("{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n");
        if let Some(body) = body {
            request.push_str("Content-Type: application/json\r\n");
            request.push_str(&format!("Content-Length: {}\r\n", body.len()));
        }
        request.push_str("\r\n");
        request.push_str(body.unwrap_or(""));
        stream.write_all(request.as_bytes()).await?;

        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).await?; // until the server closes
        parse(&raw)
    };
    tokio::time::timeout(Duration::from_secs(5), exchange)
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "no response within 5 s"))?
}

/// `GET path`.
pub async fn get(addr: SocketAddr, path: &str) -> io::Result<Response> {
    send(addr, "GET", path, None).await
}

fn parse(raw: &[u8]) -> io::Result<Response> {
    let text = String::from_utf8_lossy(raw);
    let (head, body) = text
        .split_once("\r\n\r\n")
        .ok_or_else(|| io::Error::other("response has no blank line after the head"))?;
    let mut lines = head.lines();
    let status_line = lines.next().unwrap_or_default(); // "HTTP/1.1 200 OK"
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .ok_or_else(|| io::Error::other(format!("bad status line {status_line:?}")))?;
    let headers: Vec<(String, String)> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_string(), value.trim().to_string()))
        .collect();
    let response = Response {
        status,
        headers,
        body: body.to_string(),
    };
    if response.header("transfer-encoding") == Some("chunked") {
        return Err(io::Error::other(
            "this tiny client can't read chunked bodies",
        ));
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_status_headers_and_body() {
        let raw = b"HTTP/1.1 201 Created\r\ncontent-type: application/json\r\nLocation: /api/notes/1\r\n\r\n{\"id\":1}";
        let response = parse(raw).unwrap();
        assert_eq!(response.status, 201);
        assert_eq!(response.header("Content-Type"), Some("application/json"));
        assert_eq!(response.header("location"), Some("/api/notes/1"));
        assert_eq!(response.body, "{\"id\":1}");
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse(b"hello").is_err());
        assert!(parse(b"HTTP/1.1 abc\r\n\r\n").is_err());
    }
}
