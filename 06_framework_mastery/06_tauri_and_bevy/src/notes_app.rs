//! A small "app" on the simulated Tauri layer: the Rust backend of a notes
//! window. In a real Tauri project these would be `#[tauri::command]`
//! functions in `src-tauri/src/lib.rs`, and the page would call them with
//! `invoke`.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::ipc::{Context, Router};

/// One note, as the page sees it (JSON with camelCase keys).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: u32,
    pub title: String,
    pub is_done: bool,
}

/// The app's data, managed by the router and shared by every command.
#[derive(Debug, Default)]
pub struct Notes {
    next_id: u32,
    items: Vec<Note>,
}

impl Notes {
    /// How many notes there are.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// The longest title a note may have, in characters.
pub const MAX_TITLE: usize = 80;

// ---------- argument types: what the page sends ----------
//
// Tauri turns `invoke("add_note", { title })` into these. JavaScript names
// are camelCase, so the structs say `rename_all = "camelCase"`, and
// `deny_unknown_fields` rejects anything the command doesn't expect.

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GreetArgs {
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AddNoteArgs {
    pub title: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NoteIdArgs {
    pub note_id: u32,
}

/// For commands that take no arguments: `invoke("list_notes")` sends `{}`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoArgs {}

// ---------- commands ----------

/// `invoke("greet", { name: "Ama" })`
pub fn greet(_: &Context<'_>, args: GreetArgs) -> Result<String, String> {
    Ok(format!(
        "Hello, {}! This greeting came from Rust.",
        args.name
    ))
}

/// Checks a title: never trust what the page sends (it's a web page: a bug
/// or injected script could send anything).
pub fn check_title(title: &str) -> Result<String, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("a note needs a title".into());
    }
    if title.chars().count() > MAX_TITLE {
        return Err(format!("a title can be at most {MAX_TITLE} characters"));
    }
    Ok(title.to_string())
}

/// Locks the notes, turning a poisoned lock into an error message.
pub fn notes<'a>(ctx: &'a Context<'_>) -> Result<std::sync::MutexGuard<'a, Notes>, String> {
    ctx.state
        .get::<Mutex<Notes>>()?
        .lock()
        .map_err(|_| "the notes are unavailable".to_string())
}

/// `invoke("add_note", { title: "Buy milk" })` → the new note.
pub fn add_note(ctx: &Context<'_>, args: AddNoteArgs) -> Result<Note, String> {
    let title = check_title(&args.title)?;
    let mut notes = notes(ctx)?;
    notes.next_id += 1;
    let note = Note {
        id: notes.next_id,
        title,
        is_done: false,
    };
    notes.items.push(note.clone());
    ctx.events.emit("notes-changed", notes.len());
    Ok(note)
}

/// `invoke("list_notes")` → every note.
pub fn list_notes(ctx: &Context<'_>, _: NoArgs) -> Result<Vec<Note>, String> {
    Ok(notes(ctx)?.items.clone())
}

/// `invoke("toggle_note", { noteId: 1 })` → the changed note.
pub fn toggle_note(ctx: &Context<'_>, args: NoteIdArgs) -> Result<Note, String> {
    let mut notes = notes(ctx)?;
    let note = find(&mut notes, args.note_id)?;
    note.is_done = !note.is_done;
    let note = note.clone();
    ctx.events.emit("notes-changed", notes.len());
    Ok(note)
}

/// `invoke("delete_note", { noteId: 1 })`
pub fn delete_note(ctx: &Context<'_>, args: NoteIdArgs) -> Result<(), String> {
    let mut notes = notes(ctx)?;
    find(&mut notes, args.note_id)?;
    notes.items.retain(|n| n.id != args.note_id);
    ctx.events.emit("notes-changed", notes.len());
    Ok(())
}

/// The note with `id`, or a "no note" error.
pub fn find(notes: &mut Notes, id: u32) -> Result<&mut Note, String> {
    notes
        .items
        .iter_mut()
        .find(|n| n.id == id)
        .ok_or_else(|| format!("there is no note {id}"))
}

/// The security rules, as they'd be in `src-tauri/capabilities/*.json`:
/// the main window can do everything; the small "widget" window can only
/// read.
pub const DEFAULT_CAPABILITIES: &str = r#"[
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

/// The app's router with the default capabilities.
pub fn build_router() -> Router {
    build_router_with(DEFAULT_CAPABILITIES).expect("the default capabilities are valid")
}

/// The app's router with your own capability JSON (exercise 5).
///
/// # Errors
///
/// If the capability JSON is invalid.
pub fn build_router_with(capabilities: &str) -> Result<Router, String> {
    Router::new()
        .manage(Mutex::new(Notes::default()))
        .command("greet", greet)
        .command("add_note", add_note)
        .command("list_notes", list_notes)
        .command("toggle_note", toggle_note)
        .command("delete_note", delete_note)
        .capabilities_json(capabilities)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::Frontend;
    use serde_json::json;

    #[test]
    fn add_list_toggle_delete() {
        let router = build_router();
        let mut page = Frontend::new("main");
        let note = page
            .invoke(&router, "add_note", json!({"title": "  Buy milk "}))
            .unwrap();
        assert_eq!(note, json!({"id": 1, "title": "Buy milk", "isDone": false}));
        page.invoke(&router, "add_note", json!({"title": "Walk the dog"}))
            .unwrap();

        let toggled = page
            .invoke(&router, "toggle_note", json!({"noteId": 1}))
            .unwrap();
        assert_eq!(toggled["isDone"], true);

        page.invoke(&router, "delete_note", json!({"noteId": 2}))
            .unwrap();
        let list = page.invoke(&router, "list_notes", json!({})).unwrap();
        assert_eq!(
            list,
            json!([{"id": 1, "title": "Buy milk", "isDone": true}])
        );
    }

    #[test]
    fn changes_emit_events() {
        let router = build_router();
        let mut page = Frontend::new("main");
        page.invoke(&router, "add_note", json!({"title": "a"}))
            .unwrap();
        page.invoke(&router, "add_note", json!({"title": "b"}))
            .unwrap();
        let events = router.take_events();
        let counts: Vec<_> = events
            .iter()
            .map(|e| (e.event.as_str(), e.payload.clone()))
            .collect();
        assert_eq!(
            counts,
            [("notes-changed", json!(1)), ("notes-changed", json!(2))]
        );
    }

    #[test]
    fn input_is_validated() {
        let router = build_router();
        let mut page = Frontend::new("main");
        assert_eq!(
            page.invoke(&router, "add_note", json!({"title": "   "})),
            Err("a note needs a title".into())
        );
        let long = "x".repeat(MAX_TITLE + 1);
        assert_eq!(
            page.invoke(&router, "add_note", json!({ "title": long })),
            Err("a title can be at most 80 characters".into())
        );
        let err = page
            .invoke(&router, "add_note", json!({"title": "ok", "isAdmin": true}))
            .unwrap_err();
        assert!(err.contains("unknown field `isAdmin`"), "{err}");
        assert_eq!(
            page.invoke(&router, "toggle_note", json!({"noteId": 42})),
            Err("there is no note 42".into())
        );
    }

    #[test]
    fn the_widget_window_can_only_read() {
        let router = build_router();
        let mut main = Frontend::new("main");
        let mut widget = Frontend::new("widget");
        main.invoke(&router, "add_note", json!({"title": "from main"}))
            .unwrap();
        assert_eq!(
            widget.invoke(&router, "list_notes", json!({})).unwrap()[0]["title"],
            "from main"
        );
        assert_eq!(
            widget.invoke(&router, "delete_note", json!({"noteId": 1})),
            Err("command delete_note not allowed for window widget".into())
        );
        assert_eq!(
            widget.invoke(&router, "greet", json!({"name": "Kofi"})),
            Ok(json!("Hello, Kofi! This greeting came from Rust."))
        );
    }

    #[test]
    fn every_command_is_registered() {
        assert_eq!(
            build_router().command_names(),
            [
                "add_note",
                "delete_note",
                "greet",
                "list_notes",
                "toggle_note"
            ]
        );
    }
}
