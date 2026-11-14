//! [`RequestError`]: everything that can go wrong while building a request.

use std::path::PathBuf;

use crate::{Method, UrlError};

/// Why a request couldn't be built.
///
/// One enum per library (or per area of a big library), with a variant per
/// kind of failure, so callers can `match` on what they can handle. Each
/// variant carries the details, and keeps the lower-level error as its
/// `source()`, so nothing is lost.
///
/// `#[non_exhaustive]`: new kinds of failure can be added in a minor
/// version.
///
/// It is `Send + Sync + 'static`, so it fits in a
/// `Box<dyn Error + Send + Sync>` and can cross threads (a test checks).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RequestError {
    /// The URL didn't parse. `source()` says why.
    #[error("invalid URL {url:?}")]
    InvalidUrl {
        /// The URL as written.
        url: String,
        /// Why it didn't parse.
        #[source]
        source: UrlError,
    },

    /// A header name with characters that aren't allowed.
    #[error("invalid header name {name:?}: use letters, digits and `-`")]
    InvalidHeader {
        /// The name as written.
        name: String,
    },

    /// The body file couldn't be read. `source()` is the `io::Error`.
    #[error("could not read the body from {}", path.display())]
    BodyFile {
        /// The file.
        path: PathBuf,
        /// Why it couldn't be read.
        #[source]
        source: std::io::Error,
    },

    /// A body on a request whose method doesn't take one.
    #[error("a {method} request can't have a body")]
    BodyNotAllowed {
        /// The request's method.
        method: Method,
    },
}
