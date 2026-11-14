//! # Tauri and Bevy, the ideas without the downloads
//!
//! Two big optional tracks, explained by rebuilding their core ideas in
//! plain Rust (no `tauri` or `bevy` dependency: both are large to build).
//!
//! - [`ecs`] and [`critters`]: Bevy's Entity-Component-System model.
//!   Entities are ids, components are data stored by type, systems are
//!   functions run by a schedule, resources are shared singletons.
//! - [`ipc`] and [`notes_app`]: Tauri's architecture. A web page calls Rust
//!   "commands" by sending JSON messages; the Rust side checks permissions,
//!   deserializes the arguments, runs the command with shared state, and
//!   sends back JSON or emits events.
//!
//! ```
//! use tauri_and_bevy::critters;
//!
//! let mut world = critters::new_world(42);
//! let schedule = critters::schedule();
//! for _ in 0..10 {
//!     schedule.run(&mut world); // one frame: every system once
//! }
//! assert_eq!(world.resource::<critters::Tick>().0, 10);
//! ```
//!
//! ```
//! use serde_json::json;
//! use tauri_and_bevy::{ipc::Frontend, notes_app};
//!
//! let backend = notes_app::build_router();
//! let mut page = Frontend::new("main");
//! let note = page.invoke(&backend, "add_note", json!({ "title": "Buy milk" }))?;
//! assert_eq!(note["title"], "Buy milk");
//! # Ok::<(), String>(())
//! ```

pub mod critters;
pub mod ecs;
pub mod ipc;
pub mod notes_app;
