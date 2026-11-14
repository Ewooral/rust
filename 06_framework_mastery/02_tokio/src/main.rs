//! Mini project: a TCP chat server on Tokio (the logic is in `src/lib.rs`).
//!
//!   cargo run -p tokio_runtime                         serve on 127.0.0.1:6142
//!   cargo run -p tokio_runtime -- serve 127.0.0.1:7000 serve somewhere else
//!   cargo run -p tokio_runtime -- client               chat (in another terminal)
//!   cargo run -p tokio_runtime -- client 127.0.0.1:7000
//!
//! Open two or three terminals with a client each and talk. In a client:
//! type a name first, then messages; `/who`, `/me waves`, `/quit`.
//! Ctrl+C on the server tells every client and shuts down gracefully.

use std::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio_runtime::Config;
use tokio_util::sync::CancellationToken;

const DEFAULT_ADDR: &str = "127.0.0.1:6142";

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        None | Some("serve") => serve(args.get(1).map_or(DEFAULT_ADDR, String::as_str)).await,
        Some("client") => client(args.get(1).map_or(DEFAULT_ADDR, String::as_str)).await,
        Some(other) => {
            eprintln!("unknown command {other:?}; use `serve [ADDR]` or `client [ADDR]`");
            std::process::exit(2);
        }
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

async fn serve(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("chat server on {}; Ctrl+C to stop", listener.local_addr()?);

    let shutdown = CancellationToken::new();
    let trigger = shutdown.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            println!("Ctrl+C: telling everyone and shutting down");
            trigger.cancel();
        }
    });

    tokio_runtime::run(listener, Config::default(), shutdown).await?;
    println!("every client has gone; bye");
    Ok(())
}

/// A tiny terminal client: lines you type go to the server, lines from the
/// server are printed. Two sources of input, one `select!`.
async fn client(addr: &str) -> io::Result<()> {
    let stream = TcpStream::connect(addr).await?;
    let (from_server, mut to_server) = stream.into_split();
    let mut server_lines = BufReader::new(from_server).lines();
    let mut user_lines = BufReader::new(tokio::io::stdin()).lines();
    let mut typing = true;

    loop {
        tokio::select! {
            line = server_lines.next_line() => match line? {
                Some(line) => println!("{line}"),
                None => break, // the server closed the connection
            },
            // `, if typing` switches this branch off once input has ended.
            line = user_lines.next_line(), if typing => match line? {
                Some(line) => {
                    to_server.write_all(line.as_bytes()).await?;
                    to_server.write_all(b"\n").await?;
                }
                None => {
                    // End of input (Ctrl+Z then Enter on Windows, Ctrl+D
                    // elsewhere): say we're done writing, but keep reading
                    // until the server hangs up, so its last words arrive.
                    typing = false;
                    to_server.shutdown().await?;
                }
            },
        }
    }
    Ok(())
}
