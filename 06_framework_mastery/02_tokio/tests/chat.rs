//! End to end: the real chat server on `127.0.0.1:0`, real TCP clients.
//! Every read has a 5-second limit (inside `ChatClient`), and every test
//! shuts the server down and checks that `run` returns.
//!
//! Run: cargo test -p tokio_runtime --test chat

use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_runtime::{ChatClient, Config, SHUTDOWN_NOTICE, run};
use tokio_util::sync::CancellationToken;

struct TestServer {
    addr: SocketAddr,
    shutdown: CancellationToken,
    task: JoinHandle<std::io::Result<()>>,
}

async fn start(config: Config) -> TestServer {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let shutdown = CancellationToken::new();
    let task = tokio::spawn(run(listener, config, shutdown.clone()));
    TestServer {
        addr,
        shutdown,
        task,
    }
}

impl TestServer {
    async fn stop(self) {
        self.shutdown.cancel();
        tokio::time::timeout(Duration::from_secs(5), self.task)
            .await
            .expect("run() should return within 5 s of shutdown")
            .unwrap()
            .unwrap();
    }
}

#[tokio::test]
async fn messages_reach_everyone_else() {
    let server = start(Config::default()).await;
    let (mut alice, greeting) = ChatClient::join(server.addr, "alice").await.unwrap();
    assert_eq!(greeting, "* hello alice! online: alice");

    let (mut bob, greeting) = ChatClient::join(server.addr, "bob").await.unwrap();
    assert_eq!(greeting, "* hello bob! online: alice, bob");
    assert_eq!(alice.expect_line().await.unwrap(), "* bob joined");

    bob.send("hi alice").await.unwrap();
    assert_eq!(alice.expect_line().await.unwrap(), "bob: hi alice");
    alice.send("/me waves").await.unwrap();
    assert_eq!(bob.expect_line().await.unwrap(), "* alice waves");

    // Nobody hears their own message: bob's next line is the /who answer.
    bob.send("/who").await.unwrap();
    assert_eq!(bob.expect_line().await.unwrap(), "* online: alice, bob");

    server.stop().await;
}

#[tokio::test]
async fn names_must_be_valid_and_unique() {
    let server = start(Config::default()).await;
    let (_alice, _) = ChatClient::join(server.addr, "alice").await.unwrap();

    let mut other = ChatClient::connect(server.addr).await.unwrap();
    other.expect_line().await.unwrap(); // the welcome
    other.send("alice").await.unwrap();
    assert_eq!(
        other.expect_line().await.unwrap(),
        "* alice is taken, try another"
    );
    other.send("two words").await.unwrap();
    assert_eq!(
        other.expect_line().await.unwrap(),
        "* letters, digits, - and _ only, try again"
    );
    other.send("carol").await.unwrap();
    assert_eq!(
        other.expect_line().await.unwrap(),
        "* hello carol! online: alice, carol"
    );

    server.stop().await;
}

#[tokio::test]
async fn quitting_frees_the_name_and_tells_the_others() {
    let server = start(Config::default()).await;
    let (mut alice, _) = ChatClient::join(server.addr, "alice").await.unwrap();
    let (mut bob, _) = ChatClient::join(server.addr, "bob").await.unwrap();
    assert_eq!(alice.expect_line().await.unwrap(), "* bob joined");

    bob.send("/quit").await.unwrap();
    assert_eq!(bob.expect_line().await.unwrap(), "* bye");
    assert_eq!(
        bob.recv().await.unwrap(),
        None,
        "server closes the connection"
    );
    assert_eq!(alice.expect_line().await.unwrap(), "* bob left");

    let (_bob_again, greeting) = ChatClient::join(server.addr, "bob").await.unwrap();
    assert_eq!(greeting, "* hello bob! online: alice, bob");

    server.stop().await;
}

#[tokio::test]
async fn slow_namers_and_long_lines_are_handled() {
    let config = Config {
        name_timeout: Duration::from_millis(200),
        max_line: 16,
        ..Config::default()
    };
    let server = start(config).await;

    let mut silent = ChatClient::connect(server.addr).await.unwrap();
    silent.expect_line().await.unwrap(); // the welcome
    assert_eq!(
        silent.expect_line().await.unwrap(),
        "* no name given in time, bye"
    );
    assert_eq!(silent.recv().await.unwrap(), None);

    let (mut chatty, _) = ChatClient::join(server.addr, "chatty").await.unwrap();
    chatty.send(&"x".repeat(40)).await.unwrap();
    assert_eq!(
        chatty.expect_line().await.unwrap(),
        "* line too long (max 16 bytes), bye"
    );
    assert_eq!(chatty.recv().await.unwrap(), None);

    server.stop().await;
}

#[tokio::test]
async fn shutdown_tells_every_client_and_waits_for_them() {
    let server = start(Config::default()).await;
    let (mut alice, _) = ChatClient::join(server.addr, "alice").await.unwrap();
    let mut naming = ChatClient::connect(server.addr).await.unwrap();
    naming.expect_line().await.unwrap(); // still deciding on a name

    let addr = server.addr;
    server.stop().await; // cancel, then wait for run() to return

    assert_eq!(alice.expect_line().await.unwrap(), SHUTDOWN_NOTICE);
    assert_eq!(alice.recv().await.unwrap(), None);
    assert_eq!(naming.expect_line().await.unwrap(), SHUTDOWN_NOTICE);
    assert_eq!(naming.recv().await.unwrap(), None);
    // The listener is gone, so nobody new can connect. (Windows retries a
    // refused connection for about 2 s; Linux refuses at once.)
    let late = tokio::time::timeout(Duration::from_secs(5), ChatClient::connect(addr)).await;
    assert!(matches!(late, Ok(Err(_))), "connection should be refused");
}
