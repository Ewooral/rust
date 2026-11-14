//! Solution to exercise 2: make it compile (E0004).
//!
//! This program is ANOTHER crate using the `api_design` library. It matches
//! on two library enums and lists every variant, but both enums are
//! `#[non_exhaustive]`: the library may add variants in a minor version,
//! so a match outside the library must say what to do with ones it hasn't
//! seen yet.
//!
//! Add a `_ =>` arm to each match, with a sensible answer for "a method or
//! an error this program doesn't know about": a method you don't know is
//! NOT safe to retry, and an error you don't know gets its own `Display`
//! text. Don't remove the arms that are there.
//!
//! Run: cargo run -p api_design --features solutions --bin sol02_fix_non_exhaustive_match

use api_design::{Method, Request, RequestError};

/// Safe methods only read, so a failed request can simply be sent again.
fn safe_to_retry(method: Method) -> bool {
    match method {
        Method::Get | Method::Head => true,
        Method::Post | Method::Put | Method::Delete => false,
        _ => false, // a method we don't know: don't guess that it's safe
    }
}

/// A hint for the person using the program.
fn hint(error: &RequestError) -> String {
    match error {
        RequestError::InvalidUrl { url, .. } => format!("check the address {url:?}"),
        RequestError::InvalidHeader { name } => format!("rename the header {name:?}"),
        RequestError::BodyFile { path, .. } => format!("check that {} exists", path.display()),
        RequestError::BodyNotAllowed { method } => format!("{method} can't send a body"),
        other => other.to_string(), // a newer kind of error: its message is all we know
    }
}

fn main() {
    for method in [Method::Get, Method::Post] {
        println!("{method}: safe to retry? {}", safe_to_retry(method));
    }
    let error = Request::get("htp://typo.example").unwrap_err();
    println!("hint: {}", hint(&error));
}
