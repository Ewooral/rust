//! A tiny HTTP/1.1 client over a raw TCP socket, for the examples and the
//! end-to-end tests. (A real program would use `awc` or `reqwest`; this keeps
//! the lesson's dependencies small and shows exactly what goes over the wire.)

use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// A parsed response.
#[derive(Debug, Clone)]
pub struct Response {
    /// e.g. 200.
    pub status: u16,
    /// Header names in lower case.
    pub headers: Vec<(String, String)>,
    /// The body as text.
    pub body: String,
}

impl Response {
    /// The first header with this (lower-case) name.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    /// The body parsed as JSON (`Null` if it isn't JSON).
    pub fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.body).unwrap_or(serde_json::Value::Null)
    }
}

/// Sends one request and reads the whole response. Gives up after 5 seconds,
/// so a broken server fails a test instead of hanging it.
pub async fn send(
    addr: SocketAddr,
    method: &str,
    path: &str,
    json_body: Option<&str>,
) -> io::Result<Response> {
    let exchange = async {
        let mut stream = TcpStream::connect(addr).await?;
        let mut request =
            format!("{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n");
        if let Some(body) = json_body {
            request.push_str("Content-Type: application/json\r\n");
            request.push_str(&format!("Content-Length: {}\r\n", body.len()));
        }
        request.push_str("\r\n");
        request.push_str(json_body.unwrap_or(""));
        stream.write_all(request.as_bytes()).await?;
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).await?;
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

fn bad(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.to_string())
}

fn parse(raw: &[u8]) -> io::Result<Response> {
    let text = String::from_utf8_lossy(raw);
    let (head, body) = text
        .split_once("\r\n\r\n")
        .ok_or_else(|| bad("no blank line after the headers"))?;
    let mut lines = head.lines();
    let status = lines
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .ok_or_else(|| bad("bad status line"))?;
    let headers: Vec<(String, String)> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(n, v)| (n.trim().to_ascii_lowercase(), v.trim().to_string()))
        .collect();
    let chunked = headers
        .iter()
        .any(|(n, v)| n == "transfer-encoding" && v.eq_ignore_ascii_case("chunked"));
    let body = if chunked {
        dechunk(body)?
    } else {
        body.to_string()
    };
    Ok(Response {
        status,
        headers,
        body,
    })
}

/// Undoes `Transfer-Encoding: chunked`: `<hex size>\r\n<data>\r\n` … `0\r\n\r\n`.
fn dechunk(mut rest: &str) -> io::Result<String> {
    let mut body = String::new();
    loop {
        let (size, after) = rest.split_once("\r\n").ok_or_else(|| bad("bad chunk"))?;
        let size = usize::from_str_radix(size.trim(), 16).map_err(|_| bad("bad chunk size"))?;
        if size == 0 {
            return Ok(body);
        }
        let chunk = after.get(..size).ok_or_else(|| bad("short chunk"))?;
        body.push_str(chunk);
        rest = after.get(size + 2..).ok_or_else(|| bad("short chunk"))?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_sized_and_a_chunked_response() {
        let sized = parse(b"HTTP/1.1 201 Created\r\ncontent-length: 2\r\nLocation: /a/1\r\n\r\nok")
            .unwrap();
        assert_eq!(sized.status, 201);
        assert_eq!(sized.header("location"), Some("/a/1"));
        assert_eq!(sized.body, "ok");
        let chunked =
            parse(b"HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\n\r\n3\r\nabc\r\n2\r\nde\r\n0\r\n\r\n")
                .unwrap();
        assert_eq!(chunked.body, "abcde");
    }
}
