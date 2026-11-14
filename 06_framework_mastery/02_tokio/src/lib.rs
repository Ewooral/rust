//! Mini project: a **TCP chat server** on Tokio.
//!
//! - [`protocol`]: parsing a line into a command, validating names (no I/O);
//! - [`server`]: `run`, one task per client, a `broadcast` channel to fan
//!   messages out, `select!` in every connection loop, a name timeout,
//!   graceful shutdown with `CancellationToken` + `TaskTracker`;
//! - [`client`]: a small client for the tests and the `chat_demo` example.
//!
//! `tests/chat.rs` runs the real server on `127.0.0.1:0` with real clients.

pub mod client;
pub mod protocol;
pub mod server;

pub use client::ChatClient;
pub use server::{Config, SHUTDOWN_NOTICE, run};
