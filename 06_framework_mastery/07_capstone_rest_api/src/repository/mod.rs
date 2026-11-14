//! The repository layer: the ONLY code that writes SQL.
//!
//! Each repository holds a cheap clone of the pool and returns plain domain
//! types or `sqlx::Error`. It knows nothing about HTTP, tokens or who is
//! allowed to do what: that's the service layer's job.

pub mod reviews;
pub mod users;

pub use reviews::ReviewRepository;
pub use users::{StoredUser, UserRepository};
