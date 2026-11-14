//! A small chat client for the tests and the `chat_demo` example: the same
//! `Framed` + `LinesCodec` the server uses, with a 5-second limit on every
//! read so a broken server fails a test instead of hanging it.

use futures::SinkExt;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

pub struct ChatClient {
    lines: Framed<TcpStream, LinesCodec>,
}

impl ChatClient {
    pub async fn connect(addr: SocketAddr) -> io::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(ChatClient {
            lines: Framed::new(stream, LinesCodec::new()),
        })
    }

    /// Connects, answers the name question, and returns the client with
    /// the server's greeting (`* hello <name>! online: ...`).
    pub async fn join(addr: SocketAddr, name: &str) -> io::Result<(Self, String)> {
        let mut client = Self::connect(addr).await?;
        client.expect_line().await?; // "* welcome! what is your name?"
        client.send(name).await?;
        let greeting = client.expect_line().await?;
        Ok((client, greeting))
    }

    pub async fn send(&mut self, line: &str) -> io::Result<()> {
        self.lines.send(line).await.map_err(io::Error::other)
    }

    /// The next line, or `None` when the server closed the connection.
    pub async fn recv(&mut self) -> io::Result<Option<String>> {
        match timeout(Duration::from_secs(5), self.lines.next()).await {
            Err(_) => Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "no line within 5 s",
            )),
            Ok(None) => Ok(None),
            Ok(Some(line)) => line.map(Some).map_err(io::Error::other),
        }
    }

    /// The next line; an error if the connection closed instead.
    pub async fn expect_line(&mut self) -> io::Result<String> {
        self.recv().await?.ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, "server closed the connection")
        })
    }
}
