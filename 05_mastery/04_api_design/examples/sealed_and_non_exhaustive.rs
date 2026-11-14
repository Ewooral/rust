//! Using the library from ANOTHER crate (every example is one): a sealed
//! trait you can use but not implement, and `#[non_exhaustive]` types.
//!
//! Run: cargo run -p api_design --example sealed_and_non_exhaustive

use api_design::{Body, IntoBody, Method, Parts, Request, RequestError};

/// `Method` is `#[non_exhaustive]`, so even with every variant listed, a
/// match from another crate needs `_`. Without it:
///     error[E0004]: non-exhaustive patterns: `_` not covered
///     = note: `Method` is marked as non-exhaustive, so a wildcard `_` is
///             necessary to match exhaustively
fn describe(method: Method) -> &'static str {
    match method {
        Method::Get => "read",
        Method::Head => "peek",
        Method::Post => "create",
        Method::Put => "replace",
        Method::Delete => "remove",
        _ => "something new", // a future version's PATCH lands here
    }
}

/// The same goes for the error enum: handle what you know, pass on the rest.
fn advice(error: &RequestError) -> String {
    match error {
        RequestError::InvalidUrl { url, .. } => format!("check the URL {url:?}"),
        RequestError::BodyFile { path, .. } => format!("check {}", path.display()),
        other => format!("{other}"),
    }
}

// A sealed trait can be USED…
fn body_size(body: impl IntoBody) -> usize {
    body.into_body().len()
}

// …but not implemented:
//     struct Json(String);
//     impl IntoBody for Json { … }
//     error[E0277]: the trait bound `Json: api_design::body::private::Sealed` is not satisfied
//     = note: `IntoBody` is a "sealed trait", because to implement it you also
//             need to implement `api_design::body::private::Sealed`, which is
//             not accessible; …
// That's the point: the library may add methods to `IntoBody` in a minor
// version, since nobody else has an impl that would lack them.

fn main() {
    for method in [Method::Get, Method::Post, Method::Delete] {
        println!("{method:>6}: {}", describe(method));
    }

    println!(
        "body sizes: {} {} {}",
        body_size("text"),
        body_size(vec![1u8, 2, 3]),
        body_size(Body::default())
    );

    let error = Request::get("ftp://example.org").unwrap_err();
    println!("advice: {}", advice(&error));

    // A `#[non_exhaustive]` STRUCT: other crates can read its public fields
    // and move them out, but must write `..` when destructuring, and can't
    // build one with a struct literal:
    //     let Parts { method, url, headers, body } = parts;
    //     error[E0638]: `..` required with struct marked as non-exhaustive
    //     Parts { method, url, headers, body }
    //     error[E0639]: cannot create non-exhaustive struct using struct expression
    let parts: Parts = Request::get("http://example.org/x").unwrap().into_parts();
    let Parts { method, url, .. } = parts;
    println!("parts: {method} {url}");

    // Note: `#[non_exhaustive]` on an ENUM doesn't stop other crates from
    // building its variants: `RequestError::InvalidHeader { name }` compiles
    // here. Put the attribute on a variant to stop that too.
    let made_here = RequestError::InvalidHeader { name: "x y".into() };
    println!("made outside the library: {made_here}");
}
