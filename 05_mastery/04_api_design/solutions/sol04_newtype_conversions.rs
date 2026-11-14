//! Solution to exercise 4: a validated newtype with the standard conversions (pass 4 tests).
//!
//! `Email` wraps a `String` that is known to be a plausible email address.
//! Its field is private, so `Email::parse` is the only way in, and every
//! `Email` is valid. Implement each `todo!()`:
//!
//!   parse(text)    trim spaces; then
//!                  - not exactly one `@`            -> EmailError::At
//!                  - nothing before the `@`         -> EmailError::EmptyUser
//!                  - no `.` in the part after `@`,
//!                    or it starts/ends with `.`     -> EmailError::BadDomain(domain)
//!                  Store it with the DOMAIN in lower case (the user part
//!                  keeps its case): " Ada@Example.ORG " -> "Ada@example.org".
//!   as_str         a borrowed view (free)
//!   user, domain   the parts before and after the `@` (getters: no `get_`)
//!   into_string    give the String away (free: no copy)
//!   TryFrom<&str>, FromStr       call `parse`
//!   AsRef<str>, Display          the address as text
//!   From<Email> for String       (so `let s: String = email.into();` works)
//!
//! Test: cargo test -p api_design --features solutions --bin sol04_newtype_conversions

use std::fmt;
use std::str::FromStr;

/// An email address.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Email(String);

/// Why text isn't an email address.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EmailError {
    #[error("an email address needs exactly one `@`")]
    At,
    #[error("the part before `@` is empty")]
    EmptyUser,
    #[error("the domain {0:?} needs a dot, not at either end")]
    BadDomain(String),
}

impl Email {
    /// Checks and normalises an address.
    pub fn parse(text: &str) -> Result<Email, EmailError> {
        let text = text.trim();
        let (user, domain) = text.split_once('@').ok_or(EmailError::At)?;
        if domain.contains('@') {
            return Err(EmailError::At);
        }
        if user.is_empty() {
            return Err(EmailError::EmptyUser);
        }
        if !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
            return Err(EmailError::BadDomain(domain.to_string()));
        }
        Ok(Email(format!("{user}@{}", domain.to_lowercase())))
    }

    /// The whole address, borrowed.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The part before `@`.
    pub fn user(&self) -> &str {
        // `parse` guarantees exactly one `@`, so this can't fail.
        self.0.split_once('@').map_or("", |(user, _)| user)
    }

    /// The part after `@`, in lower case.
    pub fn domain(&self) -> &str {
        self.0.split_once('@').map_or("", |(_, domain)| domain)
    }

    /// The address as a `String`, using up the `Email`.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl TryFrom<&str> for Email {
    type Error = EmailError;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        Email::parse(text)
    }
}

impl FromStr for Email {
    type Err = EmailError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Email::parse(text)
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<Email> for String {
    fn from(email: Email) -> String {
        email.0
    }
}

/// Accepts anything that can be viewed as text: `&str`, `String`, `Email`.
fn shout(text: impl AsRef<str>) -> String {
    text.as_ref().to_uppercase()
}

fn main() {
    for text in [
        " Ada@Example.ORG ",
        "no-at-sign",
        "@example.org",
        "ada@localhost",
    ] {
        match Email::parse(text) {
            Ok(email) => println!(
                "{text:?} -> {email} (user {}, domain {}), shouted: {}",
                email.user(),
                email.domain(),
                shout(&email)
            ),
            Err(error) => println!("{text:?} -> error: {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_normalises() {
        let email = Email::parse("  Grace.Hopper@Navy.MIL ").unwrap();
        assert_eq!(email.as_str(), "Grace.Hopper@navy.mil");
        assert_eq!(email.user(), "Grace.Hopper");
        assert_eq!(email.domain(), "navy.mil");
        assert_eq!(email.to_string(), "Grace.Hopper@navy.mil");
    }

    #[test]
    fn rejects_what_is_not_an_address() {
        assert_eq!(Email::parse("ada.example.org"), Err(EmailError::At));
        assert_eq!(Email::parse("a@b@example.org"), Err(EmailError::At));
        assert_eq!(Email::parse("@example.org"), Err(EmailError::EmptyUser));
        assert_eq!(
            Email::parse("ada@localhost"),
            Err(EmailError::BadDomain("localhost".into()))
        );
        assert_eq!(
            Email::parse("ada@example."),
            Err(EmailError::BadDomain("example.".into()))
        );
    }

    #[test]
    fn standard_conversions_work() {
        let a = Email::try_from("ada@example.org").unwrap();
        let b: Email = "ada@EXAMPLE.org".parse().unwrap();
        assert_eq!(a, b);
        let c: Result<Email, _> = "nope".try_into();
        assert_eq!(c, Err(EmailError::At));
        assert_eq!(shout(&a), "ADA@EXAMPLE.ORG");
        let text: String = a.into();
        assert_eq!(text, "ada@example.org");
        assert_eq!(b.into_string(), "ada@example.org");
    }

    #[test]
    fn error_messages_read_well() {
        let error = Email::parse("ada@localhost").unwrap_err();
        assert_eq!(
            error.to_string(),
            "the domain \"localhost\" needs a dot, not at either end"
        );
    }
}
