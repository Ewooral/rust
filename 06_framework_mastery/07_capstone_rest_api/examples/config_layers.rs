//! Configuration layers: defaults < file < environment, checked at start-up.
//! Run: cargo run -p capstone_rest_api --example config_layers
//!
//! `Config::from_sources` takes the file's text and a lookup function, so
//! this example can show any "environment" without changing the real one.

use std::collections::HashMap;

use capstone_rest_api::Config;

fn env_of(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> + use<> {
    let map: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    move |key: &str| map.get(key).cloned()
}

fn main() {
    let file = "\
# app.env
APP_PORT=9000
DATABASE_URL=sqlite:from-the-file.db
JWT_SECRET=file-secret-file-secret-file-secret!
LOG_FORMAT=json
";

    println!("== the file alone");
    let config = Config::from_sources(Some(file), env_of(&[])).expect("valid");
    println!(
        "port {} | db {} | log {:?}",
        config.port, config.database_url, config.log_format
    );

    println!("\n== the environment on top (it wins)");
    let env = env_of(&[("APP_PORT", "7000"), ("ADMIN_EMAILS", "Ama@Example.com")]);
    let config = Config::from_sources(Some(file), env).expect("valid");
    println!(
        "port {} | db {} | admins {:?}",
        config.port, config.database_url, config.admin_emails
    );

    println!("\n== Debug output never shows the secret");
    println!("{config:?}");

    println!("\n== mistakes are caught before the server starts");
    let cases = [
        env_of(&[("JWT_SECRET", "file-secret-file-secret-file-secret!")]),
        env_of(&[
            ("DATABASE_URL", "sqlite::memory:"),
            ("JWT_SECRET", "too-short"),
        ]),
        env_of(&[
            ("DATABASE_URL", "sqlite::memory:"),
            ("JWT_SECRET", "file-secret-file-secret-file-secret!"),
            ("TOKEN_TTL_SECONDS", "an hour"),
        ]),
        env_of(&[
            ("DATABASE_URL", "sqlite::memory:"),
            ("JWT_SECRET", "file-secret-file-secret-file-secret!"),
            ("LOG_FORMAT", "xml"),
        ]),
    ];
    for env in cases {
        match Config::from_sources(None, env) {
            Ok(_) => println!("ok?!"),
            Err(err) => println!("error: {err}"),
        }
    }
}
