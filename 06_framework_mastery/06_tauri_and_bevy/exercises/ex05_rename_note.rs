//! Exercise 5: add a command to the notes app.
//!
//! The page wants `invoke("rename_note", { noteId: 1, title: "New title" })`.
//! Adding a command to a Tauri app takes three steps, and forgetting one is
//! a classic bug:
//!
//! 1. Write the command: `rename_note` below. Check the title with
//!    `notes_app::check_title`, find the note with `notes_app::find`
//!    (lock with `notes_app::notes(ctx)`), change it, emit
//!    `"notes-changed"` with the number of notes, and return the note.
//! 2. Register it: `.command("rename_note", rename_note)` in `router()`
//!    (Tauri: add it to `generate_handler![...]`).
//! 3. Allow it: add `"rename_note"` to the MAIN window's capability in
//!    `CAPABILITIES` (and not the widget's: it's read-only).
//!
//! Test: cargo test -p tauri_and_bevy --features exercises --bin ex05_rename_note

#![allow(unused_variables, unused_imports, dead_code)]

use serde::Deserialize;
use serde_json::json;
use tauri_and_bevy::ipc::{Context, Frontend, Router};
use tauri_and_bevy::notes_app::{self, Note};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RenameArgs {
    note_id: u32,
    title: String,
}

fn rename_note(ctx: &Context<'_>, args: RenameArgs) -> Result<Note, String> {
    todo!()
}

const CAPABILITIES: &str = r#"[
  {
    "identifier": "main-window",
    "windows": ["main"],
    "commands": ["greet", "add_note", "list_notes", "toggle_note", "delete_note"]
  },
  {
    "identifier": "widget-read-only",
    "windows": ["widget"],
    "commands": ["greet", "list_notes"]
  }
]"#;

fn router() -> Router {
    notes_app::build_router_with(CAPABILITIES).expect("valid capabilities")
    // TODO: register rename_note here
}

fn main() {
    let backend = router();
    let mut page = Frontend::new("main");
    page.invoke(&backend, "add_note", json!({ "title": "Buy mlik" }))
        .expect("added");
    let renamed = page.invoke(
        &backend,
        "rename_note",
        json!({ "noteId": 1, "title": "Buy milk" }),
    );
    match renamed {
        Ok(note) => println!("rename_note -> {note}"),
        Err(message) => println!("rename_note failed: {message}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_one_note() -> (Router, Frontend) {
        let backend = router();
        let mut page = Frontend::new("main");
        page.invoke(&backend, "add_note", json!({ "title": "Buy mlik" }))
            .unwrap();
        backend.take_events(); // forget add_note's event
        (backend, page)
    }

    #[test]
    fn renames_and_trims() {
        let (backend, mut page) = with_one_note();
        let note = page
            .invoke(
                &backend,
                "rename_note",
                json!({ "noteId": 1, "title": "  Buy milk " }),
            )
            .unwrap();
        assert_eq!(
            note,
            json!({ "id": 1, "title": "Buy milk", "isDone": false })
        );
        let list = page.invoke(&backend, "list_notes", json!({})).unwrap();
        assert_eq!(list[0]["title"], "Buy milk");
    }

    #[test]
    fn emits_notes_changed() {
        let (backend, mut page) = with_one_note();
        page.invoke(
            &backend,
            "rename_note",
            json!({ "noteId": 1, "title": "x" }),
        )
        .unwrap();
        let events = backend.take_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event, "notes-changed");
        assert_eq!(events[0].payload, json!(1));
    }

    #[test]
    fn validates_like_add_note() {
        let (backend, mut page) = with_one_note();
        assert_eq!(
            page.invoke(
                &backend,
                "rename_note",
                json!({ "noteId": 1, "title": " " })
            ),
            Err("a note needs a title".into())
        );
        assert_eq!(
            page.invoke(
                &backend,
                "rename_note",
                json!({ "noteId": 9, "title": "x" })
            ),
            Err("there is no note 9".into())
        );
        let err = page
            .invoke(&backend, "rename_note", json!({ "noteId": 1, "name": "x" }))
            .unwrap_err();
        assert!(err.contains("unknown field `name`"), "{err}");
    }

    #[test]
    fn only_the_main_window_may_rename() {
        let (backend, _) = with_one_note();
        let mut widget = Frontend::new("widget");
        assert_eq!(
            widget.invoke(
                &backend,
                "rename_note",
                json!({ "noteId": 1, "title": "x" })
            ),
            Err("command rename_note not allowed for window widget".into())
        );
    }
}
