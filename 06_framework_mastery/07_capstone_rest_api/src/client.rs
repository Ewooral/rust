//! A tiny HTTP/1.1 client over a raw TCP socket, used by the `healthcheck`
//! subcommand, the examples and the end-to-end tests. Every call gives up
//! after 5 seconds, so nothing can hang.

use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// A parsed response.
#[derive(Debug, Clone)]
pub struct Response {
    /// e.g. 201.
    pub status: u16,
    /// Header names in lower case.
    pub headers: Vec<(String, String)>,
    /// The body as text.
    pub body: String,
}

impl Response {
    /// The first header with this lower-case name.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    /// The body as JSON (`Null` if it isn't JSON).
    pub fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.body).unwrap_or(serde_json::Value::Null)
    }
}

/// Sends one request: optional bearer token, optional JSON body.
pub async fn send(
    addr: SocketAddr,
    method: &str,
    path: &str,
    token: Option<&str>,
    json_body: Option<&str>,
) -> io::Result<Response> {
    let exchange = async {
        let mut stream = TcpStream::connect(addr).await?;
        let mut head = format!("{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n");
        if let Some(token) = token {
            head.push_str(&format!("Authorization: Bearer {token}\r\n"));
        }
        let body = json_body.unwrap_or("");
        if json_body.is_some() {
            head.push_str("Content-Type: application/json\r\n");
        }
        head.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
        stream.write_all(head.as_bytes()).await?;
        stream.write_all(body.as_bytes()).await?;
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).await?;
        parse(&raw)
    };
    tokio::time::timeout(Duration::from_secs(5), exchange)
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "no response within 5 s"))?
}

fn bad(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
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

fn dechunk(mut rest: &str) -> io::Result<String> {
    let mut body = String::new();
    loop {
        let (size, after) = rest.split_once("\r\n").ok_or_else(|| bad("bad chunk"))?;
        let size = usize::from_str_radix(size.trim(), 16).map_err(|_| bad("bad chunk size"))?;
        if size == 0 {
            return Ok(body);
        }
        body.push_str(after.get(..size).ok_or_else(|| bad("short chunk"))?);
        rest = after.get(size + 2..).ok_or_else(|| bad("short chunk"))?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sized_and_chunked_responses() {
        let sized =
            parse(b"HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\n\r\n{\"a\":1}")
                .unwrap();
        assert_eq!(sized.status, 404);
        assert_eq!(sized.json()["a"], 1);
        assert_eq!(sized.header("content-type"), Some("application/json"));
        let chunked =
            parse(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\nhi\r\n0\r\n\r\n")
                .unwrap();
        assert_eq!(chunked.body, "hi");
    }
}
