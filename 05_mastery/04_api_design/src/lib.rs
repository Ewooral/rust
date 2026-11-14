//! # A small HTTP request library, designed by the API Guidelines
//!
//! The mini project for "API design". It builds HTTP/1.1 requests (it
//! doesn't send them), and every public item follows a rule from the
//! [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):
//!
//! | Guideline | Where |
//! |---|---|
//! | Naming: `as_`/`to_`/`into_`, getters without `get_`, `iter` | [`Url::as_str`], [`Url::into_string`], [`Url::host`], [`Headers::iter`] |
//! | Conversions: `From`, `TryFrom`, `FromStr`, `AsRef` | [`Url`], [`Method`] |
//! | Common traits: `Debug`, `Clone`, `PartialEq`, `Hash`, `Default`, `Display` | every type |
//! | Builder, typestate | [`RequestBuilder`]: no `build` until there's a URL |
//! | Newtypes | [`Url`] (always valid), [`Body`], [`HeaderIter`] |
//! | Sealed trait | [`IntoBody`] |
//! | `#[non_exhaustive]` | [`Method`], [`RequestError`], [`UrlError`], [`Parts`] |
//! | Error design | [`RequestError`]: one enum, `source()` chains, `Send + Sync` |
//! | Flexible arguments | `impl AsRef<str>`, `impl Into<String>`, `impl AsRef<Path>` |
//! | Documentation | every item, with `# Errors` sections and tested examples |
//!
//! ```
//! use api_design::{Method, Request};
//!
//! let request = Request::builder()
//!     .method(Method::Post)
//!     .url("http://localhost:8080/users")
//!     .header("Accept", "application/json")
//!     .body("name=Ada")
//!     .build()?;
//!
//! assert_eq!(
//!     request.to_string(),
//!     "POST /users HTTP/1.1\r\n\
//!      host: localhost:8080\r\n\
//!      accept: application/json\r\n\
//!      content-length: 8\r\n\
//!      \r\n\
//!      name=Ada"
//! );
//! # Ok::<(), api_design::RequestError>(())
//! ```

// Every public item must have a doc comment: a warning (and an error in
// CI with `-D warnings`) otherwise.
#![warn(missing_docs)]

mod body;
mod error;
mod headers;
mod method;
mod request;
mod url;

// A flat public API: users write `api_design::Url`, whatever the module
// layout inside is (lesson 03/05, `pub use`).
pub use body::{Body, IntoBody};
pub use error::RequestError;
pub use headers::{HeaderIter, Headers, IntoHeaders};
pub use method::{Method, ParseMethodError};
pub use request::{HasUrl, NoUrl, Parts, Request, RequestBuilder};
pub use url::{Url, UrlError};

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn public_types_are_send_and_sync() {
        // C-SEND-SYNC: checked at compile time, so a change that loses
        // `Send` or `Sync` (say, an `Rc` field) fails this test's build.
        fn assert_send_sync<T: Send + Sync + 'static>() {}
        assert_send_sync::<Request>();
        assert_send_sync::<RequestBuilder<HasUrl>>();
        assert_send_sync::<Url>();
        assert_send_sync::<RequestError>();
        assert_send_sync::<UrlError>();
    }

    #[test]
    fn urls_are_parsed_and_normalised() {
        let url = Url::parse("HTTP://Example.ORG:80").unwrap();
        assert_eq!(url.as_str(), "http://example.org/");
        assert_eq!(url.authority(), "example.org");
        let url: Url = "https://api.example.org:8443/v1".parse().unwrap();
        assert_eq!(url.authority(), "api.example.org:8443");
        assert_eq!(String::from(url.clone()), url.into_string());
    }

    #[test]
    fn bad_urls_say_why() {
        assert_eq!(Url::parse("example.org"), Err(UrlError::MissingScheme));
        assert_eq!(
            Url::try_from("ftp://example.org"),
            Err(UrlError::UnsupportedScheme("ftp".into()))
        );
        assert_eq!(Url::parse("http://:80/"), Err(UrlError::EmptyHost));
        let port = Url::parse("http://example.org:99999/").unwrap_err();
        assert_eq!(port.to_string(), "invalid port \"99999\"");
        assert!(port.source().is_some(), "the ParseIntError is kept");
    }

    #[test]
    fn the_builder_adds_host_and_content_length() {
        let request = Request::builder()
            .method(Method::Put)
            .url("http://localhost:3000/notes/7")
            .header("X-Trace", "abc")
            .body(String::from("hello"))
            .build()
            .unwrap();
        let headers: Vec<(&str, &str)> = request.headers().iter().collect();
        assert_eq!(
            headers,
            [
                ("host", "localhost:3000"),
                ("x-trace", "abc"),
                ("content-length", "5")
            ]
        );
        assert_eq!(request.headers().iter().len(), 3, "ExactSizeIterator");
        assert_eq!(
            request.headers().iter().next_back(),
            Some(("content-length", "5"))
        );
    }

    #[test]
    fn the_first_mistake_is_reported_with_its_source() {
        let error = Request::builder()
            .header("Bad Header", "x")
            .url("nonsense")
            .build()
            .unwrap_err();
        assert!(matches!(error, RequestError::InvalidHeader { .. }));

        let error = Request::get("http://example.org:http/").unwrap_err();
        assert_eq!(
            error.to_string(),
            "invalid URL \"http://example.org:http/\""
        );
        let cause = error.source().expect("a UrlError");
        assert_eq!(cause.to_string(), "invalid port \"http\"");
        let root = cause.source().expect("a ParseIntError");
        assert_eq!(root.to_string(), "invalid digit found in string");
    }

    #[test]
    fn bodies_need_a_method_that_takes_one() {
        let error = Request::builder()
            .url("http://example.org/")
            .body("oops")
            .build()
            .unwrap_err();
        assert_eq!(error.to_string(), "a GET request can't have a body");
    }

    #[test]
    fn body_files_keep_the_io_error() {
        let error = Request::builder()
            .method(Method::Post)
            .url("http://example.org/upload")
            .body_file("this/file/does/not/exist.txt")
            .build()
            .unwrap_err();
        let RequestError::BodyFile { source, .. } = &error else {
            panic!("expected BodyFile, got {error:?}");
        };
        assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn methods_convert_both_ways() {
        for method in [
            Method::Get,
            Method::Head,
            Method::Post,
            Method::Put,
            Method::Delete,
        ] {
            assert_eq!(method.to_string().parse::<Method>(), Ok(method));
        }
        assert!("FETCH".parse::<Method>().is_err());
    }

    #[test]
    fn requests_print_as_http() {
        let request = Request::get("http://example.org/a?b=c").unwrap();
        assert_eq!(
            request.to_string(),
            "GET /a?b=c HTTP/1.1\r\nhost: example.org\r\n\r\n"
        );
        let parts = request.into_parts();
        assert_eq!(
            (
                parts.method,
                parts.url.path(),
                parts.headers.len(),
                parts.body.len()
            ),
            (Method::Get, "/a?b=c", 1, 0)
        );
    }
}
