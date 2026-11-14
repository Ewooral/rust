//! The mini project in action: the chat server on 127.0.0.1:0 with three
//! scripted clients, then a graceful shutdown.
//! Run: cargo run -p tokio_runtime --example chat_demo
//!
//! To chat yourself, run `cargo run -p tokio_runtime` in one terminal and
//! `cargo run -p tokio_runtime -- client` in two others.

use std::io;
use tokio::net::TcpListener;
use tokio_runtime::{ChatClient, Config};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let shutdown = CancellationToken::new();
    let server = tokio::spawn(tokio_runtime::run(
        listener,
        Config::default(),
        shutdown.clone(),
    ));

    let (mut ada, hello) = ChatClient::join(addr, "ada").await?;
    println!("ada   <- {hello}");
    let (mut grace, hello) = ChatClient::join(addr, "grace").await?;
    println!("grace <- {hello}");
    println!("ada   <- {}", ada.expect_line().await?);

    let mut linus = ChatClient::connect(addr).await?;
    println!("linus <- {}", linus.expect_line().await?);
    linus.send("ada").await?;
    println!("linus -> ada");
    println!("linus <- {}", linus.expect_line().await?);
    linus.send("linus").await?;
    println!("linus -> linus");
    println!("linus <- {}", linus.expect_line().await?);
    println!("ada   <- {}", ada.expect_line().await?);
    println!("grace <- {}", grace.expect_line().await?);

    grace.send("hello everyone").await?;
    println!("grace -> hello everyone");
    println!("ada   <- {}", ada.expect_line().await?);
    println!("linus <- {}", linus.expect_line().await?);

    ada.send("/me waves").await?;
    println!("ada   -> /me waves");
    println!("grace <- {}", grace.expect_line().await?);
    println!("linus <- {}", linus.expect_line().await?);

    linus.send("/quit").await?;
    println!("linus -> /quit");
    println!("linus <- {}", linus.expect_line().await?);
    println!("ada   <- {}", ada.expect_line().await?);
    grace.expect_line().await?; // "* linus left" for grace too

    grace.send("/who").await?;
    println!("grace -> /who");
    println!("grace <- {}", grace.expect_line().await?);

    println!("(server: Ctrl+C)");
    shutdown.cancel();
    println!("ada   <- {}", ada.expect_line().await?);
    println!("grace <- {}", grace.expect_line().await?);
    server.await??;
    println!("(server: run() returned, every connection closed)");
    Ok(())
}
