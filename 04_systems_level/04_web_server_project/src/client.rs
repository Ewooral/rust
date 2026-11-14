//! A tiny blocking HTTP client, so the examples and tests can play the part
//! of the browser. Every call has a timeout, so nothing can hang forever.

use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

/// How long to wait for a connection or a response.
pub const TIMEOUT: Duration = Duration::from_secs(10);

/// Sends `request` exactly as given and returns everything the server sends
/// back before it closes the connection.
pub fn send_raw(addr: SocketAddr, request: &str) -> io::Result<String> {
    let mut stream = TcpStream::connect_timeout(&addr, TIMEOUT)?;
    stream.set_read_timeout(Some(TIMEOUT))?;
    stream.write_all(request.as_bytes())?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}

/// `GET path` with the headers a browser would send at minimum.
pub fn get(addr: SocketAddr, path: &str) -> io::Result<String> {
    send_raw(
        addr,
        &format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nUser-Agent: web_server-client\r\n\r\n"),
    )
}

/// The first line of a response, e.g. `HTTP/1.1 200 OK`.
pub fn status_line(response: &str) -> &str {
    response.lines().next().unwrap_or("")
}
