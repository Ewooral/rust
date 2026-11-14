//! [`Request`] and its typestate builder, [`RequestBuilder`].

use std::fmt;
use std::marker::PhantomData;
use std::path::Path;

use crate::{Body, Headers, IntoBody, Method, RequestError, Url};

/// An HTTP request, ready to send.
///
/// Build one with [`Request::builder`], or [`Request::get`] for the simple
/// case. The fields are private and there are getters, so the library can
/// change how it stores them, and every `Request` is known to be valid.
///
/// `Display` writes the request as it would go over the wire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    method: Method,
    url: Url,
    headers: Headers,
    body: Body,
}

impl Request {
    /// Starts building a request. The builder has no URL yet, so it has no
    /// `build` method yet either: see [`RequestBuilder`].
    pub fn builder() -> RequestBuilder<NoUrl> {
        RequestBuilder {
            method: Method::Get,
            url: None,
            headers: Headers::new(),
            body: Body::default(),
            error: None,
            state: PhantomData,
        }
    }

    /// A `GET` request with no headers or body.
    ///
    /// # Errors
    ///
    /// [`RequestError::InvalidUrl`] if `url` doesn't parse.
    ///
    /// ```
    /// let request = api_design::Request::get("http://example.org/")?;
    /// assert_eq!(request.method(), api_design::Method::Get);
    /// # Ok::<(), api_design::RequestError>(())
    /// ```
    pub fn get(url: impl AsRef<str>) -> Result<Request, RequestError> {
        Request::builder().url(url).build()
    }

    /// The method.
    pub fn method(&self) -> Method {
        self.method
    }

    /// The URL.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// The headers, including the `host` header the builder adds.
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// The body (empty for most `GET`s).
    pub fn body(&self) -> &Body {
        &self.body
    }

    /// Takes the request apart. `into_` prefix: it consumes `self`, and
    /// nothing is copied.
    pub fn into_parts(self) -> Parts {
        Parts {
            method: self.method,
            url: self.url,
            headers: self.headers,
            body: self.body,
        }
    }
}

/// A request taken apart by [`Request::into_parts`].
///
/// The fields are public, to read and move out of. `#[non_exhaustive]` means
/// a later version may add fields, so other crates can't build a `Parts`
/// with a struct literal (E0639), and must write `..` when they
/// destructure one (E0638).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Parts {
    /// The method.
    pub method: Method,
    /// The URL.
    pub url: Url,
    /// The headers.
    pub headers: Headers,
    /// The body.
    pub body: Body,
}

/// The request as text, the way HTTP/1.1 sends it.
impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} HTTP/1.1\r\n", self.method, self.url.path())?;
        for (name, value) in &self.headers {
            write!(f, "{name}: {value}\r\n")?;
        }
        f.write_str("\r\n")?;
        match self.body.as_str() {
            Some(text) => f.write_str(text),
            None => write!(f, "<{} bytes of binary data>", self.body.len()),
        }
    }
}

/// Typestate: the builder doesn't have a URL yet.
#[derive(Debug)]
pub struct NoUrl;

/// Typestate: the builder has a URL, so it can `build`.
#[derive(Debug)]
pub struct HasUrl;

/// A builder for [`Request`], with the URL tracked in the TYPE.
///
/// `RequestBuilder<NoUrl>` has a `url` method but no `build`;
/// `url` returns a `RequestBuilder<HasUrl>`, which has `build`. So
/// forgetting the URL is a compile error (E0599, no method named `build`),
/// not a run-time one.
///
/// Mistakes that can only be found at run time (a bad URL, a bad header
/// name) are remembered, and [`RequestBuilder::build`] reports the first.
/// That keeps every step chainable.
///
/// ```
/// use api_design::{Method, Request};
///
/// let request = Request::builder()
///     .method(Method::Post)
///     .url("http://localhost:8080/users")
///     .header("Content-Type", "application/json")
///     .body(r#"{"name": "Ada"}"#)
///     .build()?;
/// assert_eq!(request.headers().get("content-length"), Some("15"));
/// # Ok::<(), api_design::RequestError>(())
/// ```
///
/// ```compile_fail
/// // No URL, no `build`: error[E0599]
/// let request = api_design::Request::builder().build();
/// ```
#[derive(Debug)]
#[must_use = "a builder does nothing until you call `.build()`"]
pub struct RequestBuilder<State> {
    method: Method,
    url: Option<Url>,
    headers: Headers,
    body: Body,
    error: Option<RequestError>,
    // A zero-sized marker: "this builder is in state `State`".
    state: PhantomData<State>,
}

// Methods for EVERY state.
impl<State> RequestBuilder<State> {
    /// Sets the method (the default is `GET`).
    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    /// Adds a header. A bad name is reported by `build`.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        if let Err(error) = self.headers.insert(name, value) {
            self.error.get_or_insert(error);
        }
        self
    }

    /// Sets the body: a `String`, `&str`, `Vec<u8>`, `&[u8]` or [`Body`].
    pub fn body(mut self, body: impl IntoBody) -> Self {
        self.body = body.into_body();
        self
    }

    /// Reads the body from a file. Takes `impl AsRef<Path>`, so a `&str`, a
    /// `String`, a `&Path` or a `PathBuf` all work. A failure is reported
    /// by `build`, with the `io::Error` as its `source()`.
    pub fn body_file(mut self, path: impl AsRef<Path>) -> Self {
        // A small non-generic inner function: the generic outer one is
        // copied for every type it's called with, so keep it tiny.
        fn read(path: &Path) -> Result<Body, RequestError> {
            std::fs::read(path)
                .map(IntoBody::into_body)
                .map_err(|source| RequestError::BodyFile {
                    path: path.to_path_buf(),
                    source,
                })
        }
        match read(path.as_ref()) {
            Ok(body) => self.body = body,
            Err(error) => {
                self.error.get_or_insert(error);
            }
        }
        self
    }
}

// Only for a builder WITHOUT a URL.
impl RequestBuilder<NoUrl> {
    /// Sets the URL, moving the builder to the `HasUrl` state.
    pub fn url(self, url: impl AsRef<str>) -> RequestBuilder<HasUrl> {
        let text = url.as_ref();
        let mut error = self.error;
        let url = match Url::parse(text) {
            Ok(url) => Some(url),
            Err(source) => {
                error.get_or_insert(RequestError::InvalidUrl {
                    url: text.to_string(),
                    source,
                });
                None
            }
        };
        RequestBuilder {
            method: self.method,
            url,
            headers: self.headers,
            body: self.body,
            error,
            state: PhantomData,
        }
    }
}

// Only for a builder WITH a URL.
impl RequestBuilder<HasUrl> {
    /// Finishes the request, adding `host` and (for a body) `content-length`.
    ///
    /// # Errors
    ///
    /// The first mistake made while building: [`RequestError::InvalidUrl`],
    /// [`RequestError::InvalidHeader`], [`RequestError::BodyFile`], or
    /// [`RequestError::BodyNotAllowed`] for a body on a method that
    /// doesn't take one.
    pub fn build(self) -> Result<Request, RequestError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        let url = self
            .url
            .expect("the HasUrl state always has a URL or an error");
        if !self.body.is_empty() && !self.method.allows_body() {
            return Err(RequestError::BodyNotAllowed {
                method: self.method,
            });
        }
        let mut headers = Headers::new();
        headers.insert("host", url.authority())?;
        for (name, value) in self.headers {
            headers.insert(name, value)?;
        }
        if !self.body.is_empty() {
            headers.insert("content-length", self.body.len().to_string())?;
        }
        Ok(Request {
            method: self.method,
            url,
            headers,
            body: self.body,
        })
    }
}
