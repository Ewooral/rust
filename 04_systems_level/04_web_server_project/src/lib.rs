//! web_server: the Book's multithreaded web server (chapter 21), as a
//! library the binary, the examples and the tests all share.
//!
//! - [`pool`]: `ThreadPool`, a fixed set of `Worker` threads fed jobs
//!   through a channel, with graceful shutdown in `Drop`.
//! - [`http`]: turning a request into a response. `handle_connection` works
//!   on anything that is `Read + Write`, so it is tested without a network.
//! - [`server`]: `Server` ties a `TcpListener` to the pool, and
//!   `ShutdownHandle` stops it cleanly from another thread.
//! - [`client`]: a tiny HTTP client with timeouts, for examples and tests.
//!
//! Run the real server with `cargo run -p web_server` and open
//! <http://127.0.0.1:7878>.

pub mod client;
pub mod http;
pub mod pool;
pub mod server;

pub use http::{Response, Router, handle_connection};
pub use pool::{PoolCreationError, ThreadPool};
pub use server::{Server, ShutdownHandle};
