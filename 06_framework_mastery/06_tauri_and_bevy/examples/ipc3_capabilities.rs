//! Tauri part 3: security. The page is a web page, so treat everything it
//! sends as untrusted: permissions per window, and strict argument types.
//! Run: cargo run -p tauri_and_bevy --example ipc3_capabilities

use serde_json::json;
use tauri_and_bevy::ipc::Frontend;
use tauri_and_bevy::notes_app::{self, DEFAULT_CAPABILITIES};

fn main() {
    println!(
        "capabilities (src-tauri/capabilities/*.json in a real app):\n{DEFAULT_CAPABILITIES}\n"
    );
    let backend = notes_app::build_router();

    Frontend::new("main")
        .invoke(&backend, "add_note", json!({ "title": "Secret plans" }))
        .expect("main may add notes");

    let attempts = [
        ("widget", "list_notes", json!({})),
        ("widget", "delete_note", json!({ "noteId": 1 })),
        ("devtools-injected", "list_notes", json!({})),
        ("main", "delete_everything", json!({})),
        ("main", "add_note", json!({ "title": "x", "isAdmin": true })),
    ];
    for (window, cmd, args) in attempts {
        let result = Frontend::new(window).invoke(&backend, cmd, args);
        let shown = match result {
            Ok(value) => format!("ok: {value}"),
            Err(message) => format!("REFUSED: {message}"),
        };
        println!("{window:<18} {cmd:<18} {shown}");
    }
}
