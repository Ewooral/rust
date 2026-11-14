//! Async I/O: `TcpListener` and `TcpStream`, the `AsyncReadExt` and
//! `AsyncWriteExt` helpers, splitting a stream in two, reading lines, and
//! framing with a codec.
//! Run: cargo run -p tokio_runtime --example async_io
//!
//! An echo server on 127.0.0.1:0 serves four connections, one per part,
//! then the example ends.

use futures::SinkExt;
use std::io;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    // The echo server: every byte read is written straight back.
    let server = tokio::spawn(async move {
        for _ in 0..4 {
            let (mut socket, _peer) = listener.accept().await?;
            tokio::spawn(async move {
                // split() borrows the socket as a read half and a write half
                let (mut reader, mut writer) = socket.split();
                // copy until the client says it's done writing (EOF)
                tokio::io::copy(&mut reader, &mut writer).await
            });
        }
        Ok::<(), io::Error>(())
    });

    // 1. Bytes in, bytes out. `shutdown()` closes OUR writing direction, so
    //    the server sees end-of-file, finishes copying, and closes too.
    let mut stream = TcpStream::connect(addr).await?;
    stream.write_all(b"hello, tokio").await?;
    stream.shutdown().await?;
    let mut echoed = Vec::new();
    stream.read_to_end(&mut echoed).await?;
    println!("1. echoed back: {:?}", String::from_utf8_lossy(&echoed));

    // 2. TCP is a stream of BYTES, not of messages: nothing marks where one
    //    write ended and the next began.
    let mut stream = TcpStream::connect(addr).await?;
    for word in ["one", "two", "three"] {
        stream.write_all(word.as_bytes()).await?;
    }
    stream.shutdown().await?;
    let mut echoed = String::new();
    stream.read_to_string(&mut echoed).await?;
    println!("2. three writes came back as: {echoed:?}");

    // 3. The simplest framing: one message per line. `into_split` gives
    //    OWNED halves, so the writer can move into its own task.
    let stream = TcpStream::connect(addr).await?;
    let (reader, mut writer) = stream.into_split();
    let writing = tokio::spawn(async move {
        for word in ["one", "two", "three"] {
            writer.write_all(format!("{word}\n").as_bytes()).await?;
        }
        writer.shutdown().await
    });
    let mut lines = BufReader::new(reader).lines();
    let mut got = Vec::new();
    while let Some(line) = lines.next_line().await? {
        got.push(line);
    }
    writing.await??;
    println!("3. read line by line: {got:?}");

    // 4. A codec does the framing for you. `Framed` turns a byte stream
    //    into a Stream of messages (read with `next`) and a Sink of
    //    messages (write with `send`). LinesCodec: one String per line.
    let stream = TcpStream::connect(addr).await?;
    let mut framed = Framed::new(stream, LinesCodec::new());
    // (A codec has its own error type; turn it into an io::Error for `?`.)
    framed
        .send("framed hello")
        .await
        .map_err(io::Error::other)?;
    framed.send("framed bye").await.map_err(io::Error::other)?;
    for _ in 0..2 {
        if let Some(line) = framed.next().await {
            println!(
                "4. codec gave the message {:?}",
                line.map_err(io::Error::other)?
            );
        }
    }

    server.await??;
    Ok(())
}
