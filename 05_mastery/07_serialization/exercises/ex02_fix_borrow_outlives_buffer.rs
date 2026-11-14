//! Exercise 2: make it compile (E0597).
//!
//! `Headline<'a>` borrows its text straight out of the JSON (zero-copy). That
//! is fast, but a borrow can't outlive what it borrows from. `load_latest`
//! builds the JSON in a local String, parses it, and tries to hand the
//! borrowed `Headline` back to `main`. The String is dropped at the end of
//! the block, so the Headline would point at freed memory.
//!
//! Fix it in ONE of these ways (try both, if you like):
//!   a) keep the text alive long enough: move `let text = ...` out of the
//!      block into `main`, so it lives as long as `headline`; or
//!   b) make the struct OWN its data: `title: String`, `source: String`,
//!      and drop the lifetime. That costs an allocation per field, but
//!      the struct can then go anywhere.
//!
//! Run: cargo run -p serialization --features exercises --bin ex02_fix_borrow_outlives_buffer

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Headline<'a> {
    title: &'a str,
    source: &'a str,
}

fn fetch() -> String {
    // Pretend this came from the network.
    r#"{"title": "Rust 2024 edition ships", "source": "blog.rust-lang.org"}"#.to_string()
}

fn main() {
    let headline: Headline = {
        let text = fetch();
        serde_json::from_str(&text).unwrap()
    };
    println!("{} ({})", headline.title, headline.source);
}
