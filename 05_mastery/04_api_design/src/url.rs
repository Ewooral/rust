//! [`Url`]: a newtype that can only hold a valid `http(s)://` address.

use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

/// Why a [`Url`] couldn't be parsed.
///
/// Error messages are lower case, with no full stop, so they read well
/// inside other messages ("invalid URL …: missing `http://` …").
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum UrlError {
    /// No `http://` or `https://` at the start.
    #[error("missing `http://` or `https://` at the start")]
    MissingScheme,
    /// A scheme other than http or https.
    #[error("unsupported scheme `{0}`: only http and https")]
    UnsupportedScheme(String),
    /// Nothing between `://` and the port or path.
    #[error("the host name is empty")]
    EmptyHost,
    /// A port that isn't a number from 0 to 65535. The `ParseIntError` is
    /// kept as the error's `source()`.
    #[error("invalid port {port:?}")]
    InvalidPort {
        /// The port as written.
        port: String,
        /// Why it didn't parse.
        #[source]
        source: ParseIntError,
    },
}

/// A parsed `http://` or `https://` address.
///
/// A **newtype**: the only way to get a `Url` is through [`Url::parse`] (or
/// `TryFrom`/`FromStr`, which call it), so every `Url` in a program is valid,
/// and functions that take one never need to check again. The fields are
/// private, so the representation can change without breaking anyone.
///
/// ```
/// use api_design::Url;
///
/// let url = Url::parse("https://example.org:8443/search?q=rust")?;
/// assert_eq!(url.scheme(), "https");
/// assert_eq!(url.host(), "example.org");
/// assert_eq!(url.port(), 8443);
/// assert_eq!(url.path(), "/search?q=rust");
///
/// let short = Url::parse("http://localhost")?;
/// assert_eq!((short.port(), short.path()), (80, "/"));
/// assert_eq!(short.as_str(), "http://localhost/"); // normalised
/// # Ok::<(), api_design::UrlError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Url {
    text: String,
    scheme: &'static str,
    host: String,
    port: u16,
    path: String,
}

impl Url {
    /// Parses and checks a URL.
    ///
    /// # Errors
    ///
    /// A [`UrlError`] saying what is wrong: no scheme, an unsupported
    /// scheme, an empty host, or a port that isn't a number up to 65535.
    pub fn parse(text: &str) -> Result<Url, UrlError> {
        let text = text.trim();
        let (scheme, rest) = text.split_once("://").ok_or(UrlError::MissingScheme)?;
        let (scheme, default_port) = match scheme.to_ascii_lowercase().as_str() {
            "http" => ("http", 80),
            "https" => ("https", 443),
            _ => return Err(UrlError::UnsupportedScheme(scheme.to_string())),
        };
        let (authority, path) = match rest.find('/') {
            Some(slash) => rest.split_at(slash),
            None => (rest, "/"),
        };
        let (host, port) = match authority.rsplit_once(':') {
            Some((host, port_text)) => {
                let port = port_text.parse().map_err(|source| UrlError::InvalidPort {
                    port: port_text.to_string(),
                    source,
                })?;
                (host, port)
            }
            None => (authority, default_port),
        };
        if host.is_empty() {
            return Err(UrlError::EmptyHost);
        }
        let host = host.to_ascii_lowercase();
        let shown_port = if port == default_port {
            String::new()
        } else {
            format!(":{port}")
        };
        Ok(Url {
            text: format!("{scheme}://{host}{shown_port}{path}"),
            scheme,
            host,
            port,
            path: path.to_string(),
        })
    }

    /// `"http"` or `"https"`.
    pub fn scheme(&self) -> &str {
        self.scheme
    }

    /// The host name, in lower case. (A getter is named after the field:
    /// `host()`, not `get_host()`.)
    pub fn host(&self) -> &str {
        &self.host
    }

    /// The port: the one written, or 80 for http and 443 for https.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// The path and query, always starting with `/`.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// `host`, plus `:port` when it isn't the default: what goes in a
    /// request's `Host` header.
    pub fn authority(&self) -> String {
        let default = if self.scheme == "https" { 443 } else { 80 };
        if self.port == default {
            self.host.clone()
        } else {
            format!("{}:{}", self.host, self.port)
        }
    }

    /// The whole URL, borrowed: free.
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// The whole URL as an owned `String`, giving up the `Url`: free too,
    /// because the `String` is moved out, not copied.
    pub fn into_string(self) -> String {
        self.text
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

impl FromStr for Url {
    type Err = UrlError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Url::parse(text)
    }
}

/// `Url::try_from("http://…")`, and so `"http://…".try_into()`.
impl TryFrom<&str> for Url {
    type Error = UrlError;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        Url::parse(text)
    }
}

impl TryFrom<String> for Url {
    type Error = UrlError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Url::parse(&text)
    }
}

/// Lets a `&Url` go wherever an `impl AsRef<str>` is accepted.
impl AsRef<str> for Url {
    fn as_ref(&self) -> &str {
        &self.text
    }
}

/// `String::from(url)`, and so `let s: String = url.into();`. Implement
/// `From`; `Into` then comes for free.
impl From<Url> for String {
    fn from(url: Url) -> String {
        url.text
    }
}
