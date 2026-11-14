//! The finished `UserService` trait, its data types and the `BoxFuture` alias.

use std::fmt;
use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;

/// A pinned, boxed, sendable future that may borrow for `'a`.
///
/// This is the return type `#[async_trait]` writes for every `async fn`, given
/// a short name. The `futures` crate has the same alias,
/// `futures::future::BoxFuture`.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// A user, as every backend returns it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    /// The user's id.
    pub id: u32,
    /// The user's display name.
    pub name: String,
}

impl User {
    /// Makes a user from an id and a name.
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        User {
            id,
            name: name.into(),
        }
    }
}

/// Why a lookup failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LookupError {
    /// The backend answered: there is no user with this id.
    NotFound(u32),
    /// The backend (named) could not be reached.
    Unavailable(String),
    /// The backend (named) took longer than the time limit.
    TimedOut(String),
}

impl LookupError {
    /// `true` when asking another backend might help (it was down or slow),
    /// `false` when the answer was a real "no such user".
    pub fn is_retryable(&self) -> bool {
        !matches!(self, LookupError::NotFound(_))
    }
}

impl fmt::Display for LookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LookupError::NotFound(id) => write!(f, "no user with id {id}"),
            LookupError::Unavailable(backend) => write!(f, "{backend} is unavailable"),
            LookupError::TimedOut(backend) => write!(f, "{backend} timed out"),
        }
    }
}

impl std::error::Error for LookupError {}

/// Anything that can look users up.
///
/// `#[async_trait]` turns each `async fn` into a plain `fn` returning a
/// [`BoxFuture`], so the trait is dyn compatible (`Arc<dyn UserService>`).
/// `Send + Sync` let one service be shared by many `tokio::spawn`ed tasks.
#[async_trait]
pub trait UserService: Send + Sync {
    /// Looks up one user by id.
    async fn get_user(&self, id: u32) -> Result<User, LookupError>;

    /// A short description of this service and what it wraps, such as
    /// `cache(memory)`. A plain `fn`: `#[async_trait]` leaves it alone.
    fn describe(&self) -> String;
}
