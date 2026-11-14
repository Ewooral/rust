//! Two short demos: Bevy's ECS idea and Tauri's IPC idea, in plain Rust.
//!
//!   cargo run -p tauri_and_bevy            both
//!   cargo run -p tauri_and_bevy -- ecs     the critters simulation
//!   cargo run -p tauri_and_bevy -- ipc     a notes "app" driven by JSON messages

use std::env;
use std::process::ExitCode;

use serde_json::json;
use tauri_and_bevy::critters::{self, Log};
use tauri_and_bevy::ipc::Frontend;
use tauri_and_bevy::notes_app;

fn ecs_demo() {
    println!("=== Bevy's idea: entities, components, systems ===");
    let mut world = critters::new_world(42);
    let schedule = critters::schedule();
    println!("systems, in order: {}\n", schedule.names().join(" -> "));
    print!("{}", critters::render(&world));
    for _ in 0..3 {
        for _ in 0..10 {
            schedule.run(&mut world);
        }
        println!();
        print!("{}", critters::render(&world));
    }
    println!("\nwhat happened:");
    for line in world.resource::<Log>().0.iter() {
        println!("  {line}");
    }
}

fn ipc_demo() {
    println!("=== Tauri's idea: the page invokes Rust commands over JSON ===");
    let backend = notes_app::build_router();
    let mut main = Frontend::new("main");
    let mut widget = Frontend::new("widget");

    let calls = [
        ("main", "greet", json!({ "name": "Ama" })),
        ("main", "add_note", json!({ "title": "Buy milk" })),
        ("main", "add_note", json!({ "title": "" })),
        ("main", "toggle_note", json!({ "noteId": 1 })),
        ("widget", "list_notes", json!({})),
        ("widget", "delete_note", json!({ "noteId": 1 })),
    ];
    for (window, cmd, args) in calls {
        let page = if window == "main" {
            &mut main
        } else {
            &mut widget
        };
        let message = page.invoke_message(cmd, args);
        let reply = backend.handle_json(&message);
        println!("page -> rust: {message}");
        println!("rust -> page: {reply}\n");
    }
    for event in backend.take_events() {
        println!(
            "event to the page: {}",
            serde_json::to_string(&event).unwrap()
        );
    }
}

fn main() -> ExitCode {
    match env::args().nth(1).as_deref() {
        None => {
            ecs_demo();
            println!();
            ipc_demo();
        }
        Some("ecs") => ecs_demo(),
        Some("ipc") => ipc_demo(),
        Some(other) => {
            eprintln!("unknown demo {other:?}: use `ecs` or `ipc`");
            return ExitCode::from(2);
        }
    }
    ExitCode::SUCCESS
}
