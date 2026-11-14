//! The Book's blog post workflow (18.3), written two ways so you can compare:
//!
//! - [`dynamic`]: the state pattern. One `Post` type holds its current state
//!   as a `Box<dyn State>` and changes it at run time.
//! - [`typed`]: states as types. `DraftPost`, `PendingReviewPost` and `Post`
//!   are different types, so the compiler rejects invalid steps.

pub mod dynamic;
pub mod typed;
