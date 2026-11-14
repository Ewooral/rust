//! The service layer: the application's rules.
//!
//! Services validate input, decide who may do what (`policy`), call the
//! repositories, and speak `AppError`. They know nothing about HTTP
//! requests or JSON, so they could sit behind a CLI or a queue consumer too.

pub mod auth;
pub mod policy;
pub mod reviews;

pub use auth::AuthService;
pub use reviews::ReviewService;
