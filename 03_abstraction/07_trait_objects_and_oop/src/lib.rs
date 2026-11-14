//! Library for the chapter 18 mini project, a blog dashboard.
//!
//! - [`gui`]: a tiny text-mode GUI library. `Draw` is the shared behaviour,
//!   and a `Screen` holds `Box<dyn Draw>`, components of any type (Book 18.2).
//! - [`blog`]: the Book's post workflow, written as a state pattern with
//!   trait objects AND as states encoded in types (Book 18.3).
//!
//! Every field is private unless it is meant to be set by callers: that's the
//! encapsulation half of Book 18.1.

pub mod blog;
pub mod gui;
