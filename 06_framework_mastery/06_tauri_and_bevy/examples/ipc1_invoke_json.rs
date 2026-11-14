//! Tauri part 1: a command call is a JSON message there and a JSON message
//! back. This shows each step that `#[tauri::command]` and `invoke` hide.
//! Run: cargo run -p tauri_and_bevy --example ipc1_invoke_json

use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri_and_bevy::ipc::{Context, Frontend, Router};

/// The arguments. JavaScript sends `{ widthCm: 3, heightCm: 4 }`, Rust
/// names them `width_cm`: `rename_all = "camelCase"` translates (Tauri does
/// this for you for command parameters).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AreaArgs {
    width_cm: f64,
    height_cm: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Area {
    square_cm: f64,
    is_square: bool,
}

/// In a real app:
///     #[tauri::command]
///     fn area(width_cm: f64, height_cm: f64) -> Result<Area, String>
fn area(_: &Context<'_>, args: AreaArgs) -> Result<Area, String> {
    if args.width_cm < 0.0 || args.height_cm < 0.0 {
        return Err("sizes can't be negative".into());
    }
    Ok(Area {
        square_cm: args.width_cm * args.height_cm,
        is_square: args.width_cm == args.height_cm,
    })
}

fn main() {
    // In a real app: tauri::Builder::default().invoke_handler(tauri::generate_handler![area])
    let backend = Router::new()
        .command("area", area)
        .capabilities_json(r#"[{"identifier": "main", "windows": ["main"], "commands": ["area"]}]"#)
        .expect("valid capabilities");

    // In the page: const result = await invoke("area", { widthCm: 3, heightCm: 4 });
    let mut page = Frontend::new("main");
    let message = page.invoke_message("area", json!({ "widthCm": 3, "heightCm": 4 }));
    println!("1. the page sends:     {message}");
    let reply = backend.handle_json(&message);
    println!("2. Rust replies:       {reply}");
    println!("   (the promise resolves with the \"ok\" value)\n");

    let cases = [
        ("negative size", json!({ "widthCm": -1, "heightCm": 4 })),
        ("missing argument", json!({ "widthCm": 3 })),
        ("wrong type", json!({ "widthCm": "three", "heightCm": 4 })),
        ("snake_case keys", json!({ "width_cm": 3, "height_cm": 4 })),
    ];
    for (label, args) in cases {
        let reply = backend.handle_json(&page.invoke_message("area", args));
        println!("{label:<17} -> {reply}");
    }
    println!("\n(every \"error\" rejects the promise: catch it in JavaScript)");
}
