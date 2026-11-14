//! [`Headers`]: an ordered list of header names and values, and its iterator.

use std::iter::FusedIterator;

use crate::RequestError;

/// Request headers, in the order they were added. Names are stored in
/// lower case and compared without regard to case.
///
/// ```
/// use api_design::Headers;
///
/// let mut headers = Headers::new();
/// headers.insert("Accept", "text/html")?;
/// headers.insert("X-Request-Id", "42")?;
/// assert_eq!(headers.get("accept"), Some("text/html"));
///
/// // `iter()` borrows; `for … in &headers` does the same.
/// let names: Vec<&str> = headers.iter().map(|(name, _)| name).collect();
/// assert_eq!(names, ["accept", "x-request-id"]);
/// # Ok::<(), api_design::RequestError>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Headers {
    entries: Vec<(String, String)>,
}

impl Headers {
    /// No headers.
    pub fn new() -> Self {
        Headers::default()
    }

    /// Adds a header, or replaces the value of one with the same name.
    /// Returns the old value, if there was one.
    ///
    /// Takes `impl Into<String>`, so callers can pass a `&str` (copied) or
    /// a `String` they no longer need (moved, no copy).
    ///
    /// # Errors
    ///
    /// [`RequestError::InvalidHeader`] if the name is empty or has anything
    /// but ASCII letters, digits and `-`.
    pub fn insert(
        &mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Option<String>, RequestError> {
        let mut name = name.into();
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(RequestError::InvalidHeader { name });
        }
        name.make_ascii_lowercase();
        let value = value.into();
        match self
            .entries
            .iter_mut()
            .find(|(existing, _)| *existing == name)
        {
            Some((_, old)) => Ok(Some(std::mem::replace(old, value))),
            None => {
                self.entries.push((name, value));
                Ok(None)
            }
        }
    }

    /// The value of a header, by name in any letter case.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(existing, _)| existing.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    /// `true` if there is a header with this name.
    pub fn contains(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// How many headers there are.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` if there are none.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The headers as `(name, value)` pairs, in order.
    ///
    /// Returns a NAMED iterator type, [`HeaderIter`], rather than
    /// `impl Iterator`: users can store it in a struct field, and it can
    /// promise more (`ExactSizeIterator`, `DoubleEndedIterator`).
    pub fn iter(&self) -> HeaderIter<'_> {
        HeaderIter {
            inner: self.entries.iter(),
        }
    }
}

/// The iterator returned by [`Headers::iter`].
///
/// Its field is private: it wraps a `slice::Iter` today, and could wrap
/// something else tomorrow without a breaking change.
#[derive(Debug, Clone)]
pub struct HeaderIter<'a> {
    inner: std::slice::Iter<'a, (String, String)>,
}

impl<'a> Iterator for HeaderIter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(name, value)| (name.as_str(), value.as_str()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl DoubleEndedIterator for HeaderIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner
            .next_back()
            .map(|(name, value)| (name.as_str(), value.as_str()))
    }
}

impl ExactSizeIterator for HeaderIter<'_> {}
impl FusedIterator for HeaderIter<'_> {}

/// `for (name, value) in &headers { … }`
impl<'a> IntoIterator for &'a Headers {
    type Item = (&'a str, &'a str);
    type IntoIter = HeaderIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// `for (name, value) in headers { … }`: owned `String`s, the `Headers` is used up.
impl IntoIterator for Headers {
    type Item = (String, String);
    type IntoIter = IntoHeaders;

    fn into_iter(self) -> Self::IntoIter {
        IntoHeaders {
            inner: self.entries.into_iter(),
        }
    }
}

/// The owning iterator from `Headers::into_iter`. Also a newtype, for the
/// same reason as [`HeaderIter`].
#[derive(Debug)]
pub struct IntoHeaders {
    inner: std::vec::IntoIter<(String, String)>,
}

impl Iterator for IntoHeaders {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for IntoHeaders {}
