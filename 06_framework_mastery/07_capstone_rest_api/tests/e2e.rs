//! End-to-end tests over real TCP on `127.0.0.1:0`.
//!
//! 1. The app served in-process, stopped with a graceful shutdown signal.
//! 2. The compiled binary itself, configured ONLY through environment
//!    variables set on the child process (never on this test process), the
//!    way Docker or a process manager would run it.

use std::io::{BufRead, BufReader};
use std::net::SocketAddr;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use capstone_rest_api::{Config, build_app, client, serve};
use tokio::net::TcpListener;

#[tokio::test]
async fn a_whole_session_over_tcp_then_a_graceful_shutdown() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = build_app(Config::for_tests()).await.unwrap();
    let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(serve(listener, app, async {
        let _ = stopped.await;
    }));

    let register =
        r#"{"email":"ama@example.com","password":"correct horse battery","display_name":"Ama"}"#;
    let created = client::send(addr, "POST", "/api/v1/auth/register", None, Some(register))
        .await
        .unwrap();
    assert_eq!(created.status, 201, "{}", created.body);

    let login = r#"{"email":"ama@example.com","password":"correct horse battery"}"#;
    let login = client::send(addr, "POST", "/api/v1/auth/login", None, Some(login))
        .await
        .unwrap();
    let token = login.json()["access_token"].as_str().unwrap().to_string();

    let review = r#"{"book_title":"Emma","rating":5}"#;
    let posted = client::send(addr, "POST", "/api/v1/reviews", Some(&token), Some(review))
        .await
        .unwrap();
    assert_eq!(posted.status, 201);
    assert_eq!(posted.header("location"), Some("/api/v1/reviews/1"));
    assert!(posted.header("x-request-id").is_some());

    let page = client::send(addr, "GET", "/api/v1/reviews", None, None)
        .await
        .unwrap();
    assert_eq!(page.json()["total"], 1);

    stop.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(5), server)
        .await
        .expect("the server stops within 5 s")
        .unwrap()
        .unwrap();
}

/// Kills the child process when dropped, even if the test panics, so no
/// server is ever left running.
struct Running(Child);

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn binary() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_capstone_rest_api"));
    command
        .env_remove("APP_CONFIG_FILE")
        .env_remove("RUST_LOG")
        .env("APP_HOST", "127.0.0.1")
        .env("DATABASE_URL", "sqlite::memory:")
        .env("JWT_SECRET", "e2e-test-secret-e2e-test-secret-1234")
        .env("PASSWORD_MEMORY_KIB", "8")
        .env("PASSWORD_ITERATIONS", "1")
        .env("LOG_FORMAT", "json");
    command
}

/// Starts the binary on port 0 and reads its JSON log until it says where it
/// is listening.
fn start_binary() -> (Running, SocketAddr) {
    let mut child = binary()
        .env("APP_PORT", "0")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("the binary starts");
    let stdout = child.stdout.take().unwrap();
    let running = Running(child);
    let (found, wait) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if let Some(rest) = line.split("listening on http://").nth(1) {
                let addr: String = rest
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == ':')
                    .collect();
                let _ = found.send(addr);
            }
        }
    });
    let addr = wait
        .recv_timeout(Duration::from_secs(30))
        .expect("the server logs its address within 30 s");
    (running, addr.parse().unwrap())
}

#[tokio::test]
async fn the_binary_runs_from_environment_variables_alone() {
    let (_running, addr) = start_binary();

    let health = client::send(addr, "GET", "/health", None, None)
        .await
        .unwrap();
    assert_eq!(health.status, 200);
    assert_eq!(health.json()["database"], "ok");

    // The `healthcheck` subcommand the Docker image uses.
    let status = binary()
        .arg("healthcheck")
        .env("APP_PORT", addr.port().to_string())
        .status()
        .unwrap();
    assert!(status.success(), "healthcheck should pass: {status}");
}

#[test]
fn the_binary_refuses_to_start_without_a_secret() {
    let output = binary().env_remove("JWT_SECRET").output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("configuration error: JWT_SECRET is not set"),
        "{stderr}"
    );
}
