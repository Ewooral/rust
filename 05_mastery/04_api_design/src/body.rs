//! [`Body`] and the sealed [`IntoBody`] trait.

/// The bytes sent after the headers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Body(Vec<u8>);

impl Body {
    /// The bytes, borrowed.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// The body as text, if it is valid UTF-8.
    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.0).ok()
    }

    /// The number of bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// `true` for an empty body.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The bytes, owned. The `Body` is used up, so nothing is copied.
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

/// The trick that makes [`IntoBody`] "sealed": a public trait in a PRIVATE
/// module. Other crates can see `IntoBody`, and use it, but they can't name
/// `private::Sealed`, so they can't implement it, and so they can't
/// implement `IntoBody` either.
mod private {
    pub trait Sealed {}

    impl Sealed for String {}
    impl Sealed for &str {}
    impl Sealed for Vec<u8> {}
    impl Sealed for &[u8] {}
    impl Sealed for super::Body {}
}

/// Things that can be a request body: `String`, `&str`, `Vec<u8>`, `&[u8]`
/// and [`Body`].
///
/// This trait is **sealed**: you can use it, but not implement it for your
/// own types. That lets this library add methods to it later without
/// breaking anyone, because nobody else has an `impl` that would be missing
/// the new method.
pub trait IntoBody: private::Sealed {
    /// Converts into a [`Body`].
    fn into_body(self) -> Body;
}

impl IntoBody for String {
    fn into_body(self) -> Body {
        Body(self.into_bytes())
    }
}

impl IntoBody for &str {
    fn into_body(self) -> Body {
        Body(self.as_bytes().to_vec())
    }
}

impl IntoBody for Vec<u8> {
    fn into_body(self) -> Body {
        Body(self)
    }
}

impl IntoBody for &[u8] {
    fn into_body(self) -> Body {
        Body(self.to_vec())
    }
}

impl IntoBody for Body {
    fn into_body(self) -> Body {
        self
    }
}
