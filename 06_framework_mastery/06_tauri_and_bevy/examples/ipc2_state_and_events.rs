//! Tauri part 2: managed state shared by commands, and events pushed from
//! Rust to the page without being asked.
//! Run: cargo run -p tauri_and_bevy --example ipc2_state_and_events

use serde_json::json;
use tauri_and_bevy::ipc::Frontend;
use tauri_and_bevy::notes_app;

fn main() {
    // notes_app::build_router() does what a Tauri app's run() does:
    //   tauri::Builder::default()
    //       .manage(Mutex::new(Notes::default()))
    //       .invoke_handler(tauri::generate_handler![greet, add_note, list_notes, ...])
    let backend = notes_app::build_router();
    let mut page = Frontend::new("main");

    // Each command finds the same Mutex<Notes> through its context, like a
    // Tauri command parameter `notes: State<'_, Mutex<Notes>>`.
    for title in ["Buy milk", "Water the plants", "Call Kofi"] {
        match page.invoke(&backend, "add_note", json!({ "title": title })) {
            Ok(note) => println!("add_note -> {note}"),
            Err(message) => println!("add_note failed: {message}"),
        }
    }
    page.invoke(&backend, "toggle_note", json!({ "noteId": 2 }))
        .expect("note 2 exists");
    let list = page
        .invoke(&backend, "list_notes", json!({}))
        .expect("listing works");
    println!(
        "\nlist_notes ->\n{}",
        serde_json::to_string_pretty(&list).unwrap()
    );

    // Every change also emitted "notes-changed". In the page:
    //   await listen("notes-changed", (event) => refreshList(event.payload));
    // so a second window showing the same notes can update itself.
    println!("\nevents waiting for the page's listeners:");
    for event in backend.take_events() {
        println!("  {} with payload {}", event.event, event.payload);
    }
}
