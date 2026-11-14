//! [`Method`]: the HTTP verb.

use std::fmt;
use std::str::FromStr;

/// An HTTP request method.
///
/// `#[non_exhaustive]`: a later version may add methods (`PATCH`,
/// `OPTIONS`, …) without a breaking change, so code in other crates that
/// matches on a `Method` must have a `_ =>` arm.
///
/// ```
/// use api_design::Method;
///
/// let method: Method = "post".parse().unwrap();
/// assert_eq!(method, Method::Post);
/// assert_eq!(method.to_string(), "POST");
/// assert_eq!(Method::default(), Method::Get);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[non_exhaustive]
pub enum Method {
    /// Fetch a resource.
    #[default]
    Get,
    /// Like `GET`, without the body.
    Head,
    /// Create something, or send data to be processed.
    Post,
    /// Replace a resource.
    Put,
    /// Remove a resource.
    Delete,
}

impl Method {
    /// The method's name as it appears in a request: `"GET"`.
    ///
    /// `as_` prefix: a free, borrowed view (here even `'static`).
    pub fn as_str(self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Head => "HEAD",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
        }
    }

    /// `true` if requests with this method may carry a body.
    pub fn allows_body(self) -> bool {
        matches!(self, Method::Post | Method::Put)
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The error from parsing a [`Method`] from text.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown HTTP method {0:?}")]
pub struct ParseMethodError(String);

impl FromStr for Method {
    type Err = ParseMethodError;

    /// Accepts any letter case: `"get"`, `"Get"`, `"GET"`.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text.to_ascii_uppercase().as_str() {
            "GET" => Ok(Method::Get),
            "HEAD" => Ok(Method::Head),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            _ => Err(ParseMethodError(text.to_string())),
        }
    }
}
