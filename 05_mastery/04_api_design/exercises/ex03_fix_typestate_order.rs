//! Exercise 3: make it compile (E0599).
//!
//! `RequestBuilder` keeps track of the URL in its TYPE: a
//! `RequestBuilder<NoUrl>` has a `url` method but no `build`, and `url`
//! returns a `RequestBuilder<HasUrl>`, which has `build` but no `url`.
//!
//! Two calls below are in an impossible order. Read the errors (they even
//! say which state has the method), and fix the ORDER of the calls: every
//! request needs exactly one `.url(…)`, somewhere before `.build()`. Don't
//! change the library.
//!
//! Run: cargo run -p api_design --features exercises --bin ex03_fix_typestate_order

use api_design::{Method, Request};

fn main() {
    // A login request: method, headers, body... and then the URL.
    let login = Request::builder()
        .method(Method::Post)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("user=ada&password=hunter2")
        .build()
        .url("https://example.org/login");

    // Two URLs? The second would silently win in an ordinary builder.
    let profile = Request::builder()
        .url("https://example.org/me")
        .header("Accept", "application/json")
        .url("https://example.org/profile")
        .build();

    for request in [login, profile] {
        match request {
            Ok(request) => println!("{request}\n"),
            Err(error) => println!("error: {error}"),
        }
    }
}
