//! A guided tour of the finished API over real HTTP, then a graceful shutdown.
//! Run: cargo run -p capstone_rest_api --example api_tour
//!
//! Starts the server on a free port with the test configuration (in-memory
//! database), plays a short session as two users and an admin, and stops.

use capstone_rest_api::{Config, build_app, client, serve};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let app = build_app(Config::for_tests()).await?;
    let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(serve(listener, app, async {
        let _ = stopped.await;
    }));
    println!("server on http://{addr}\n");

    // A small helper that prints each exchange.
    let call = |method: &'static str,
                path: String,
                token: Option<String>,
                body: Option<&'static str>| async move {
        let response = client::send(addr, method, &path, token.as_deref(), body).await?;
        let shown = if response.body.len() > 110 {
            format!("{}…", &response.body[..110])
        } else {
            response.body.clone()
        };
        println!("{method:<6} {path:<26} -> {} {shown}", response.status);
        Ok::<_, std::io::Error>(response)
    };

    let password = r#""password":"correct horse battery""#;
    for (email, name) in [
        ("ama@example.com", "Ama"),
        ("kofi@example.com", "Kofi"),
        ("admin@example.com", "Admin"),
    ] {
        let body = format!(r#"{{"email":"{email}",{password},"display_name":"{name}"}}"#);
        client::send(addr, "POST", "/api/v1/auth/register", None, Some(&body)).await?;
    }
    let mut tokens = Vec::new();
    for email in ["ama@example.com", "kofi@example.com", "admin@example.com"] {
        let body = format!(r#"{{"email":"{email}",{password}}}"#);
        let login = client::send(addr, "POST", "/api/v1/auth/login", None, Some(&body)).await?;
        tokens.push(
            login.json()["access_token"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        );
    }
    let [ama, kofi, admin] = [tokens[0].clone(), tokens[1].clone(), tokens[2].clone()];
    println!("registered and logged in: Ama, Kofi and Admin\n");

    call("GET", "/health".into(), None, None).await?;
    call("GET", "/api/v1/me".into(), None, None).await?;
    call("GET", "/api/v1/me".into(), Some(ama.clone()), None).await?;
    call(
        "POST",
        "/api/v1/reviews".into(),
        Some(ama.clone()),
        Some(r#"{"book_title":"Emma","rating":5,"body":"Clever and funny."}"#),
    )
    .await?;
    call(
        "POST",
        "/api/v1/reviews".into(),
        Some(kofi.clone()),
        Some(r#"{"book_title":"Dracula","rating":4}"#),
    )
    .await?;
    call(
        "POST",
        "/api/v1/reviews".into(),
        Some(kofi.clone()),
        Some(r#"{"book_title":"","rating":7}"#),
    )
    .await?;
    call(
        "GET",
        "/api/v1/reviews?per_page=1&page=2".into(),
        None,
        None,
    )
    .await?;
    call(
        "PATCH",
        "/api/v1/reviews/1".into(),
        Some(kofi.clone()),
        Some(r#"{"rating":1}"#),
    )
    .await?;
    call("DELETE", "/api/v1/reviews/1".into(), Some(admin), None).await?;
    call("GET", "/api/v1/reviews/1".into(), None, None).await?;

    stop.send(()).ok();
    server.await??;
    println!("\nserver stopped gracefully");
    Ok(())
}
