//! Error messages: what went wrong, and WHERE (line and column).
//! Run: cargo run -p serialization --example error_messages
//!
//! 1. serde_json: line(), column(), classify()   2. pointing at the line
//! 3. TOML's built-in snippets   4. when the position is lost

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct Server {
    host: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind")]
#[allow(dead_code)]
enum Job {
    Backup { folder: String },
}

fn main() {
    // ---- 1. serde_json ------------------------------------------------------------------
    // Each error has a category: Syntax (not JSON at all), Data (JSON, but the
    // wrong shape for the type), Eof (cut short) or Io (the reader failed).
    let cases = [
        ("syntax", "{\n  \"host\": \"nas\",\n  \"port\": 80,\n}"),
        (
            "wrong type",
            "{\n  \"host\": \"nas\",\n  \"port\": \"80\"\n}",
        ),
        (
            "out of range",
            "{\n  \"host\": \"nas\",\n  \"port\": 70000\n}",
        ),
        ("missing", "{\n  \"host\": \"nas\"\n}"),
        (
            "unknown field",
            "{\n  \"host\": \"nas\",\n  \"prot\": 80\n}",
        ),
        ("cut short", "{\n  \"host\": \"nas\",\n  \"port\": 8"),
    ];
    println!("1. serde_json errors:");
    for (name, text) in cases {
        let err = serde_json::from_str::<Server>(text).unwrap_err();
        let category = format!("{:?}", err.classify());
        println!("  {name:<14} {category:<7} {err}");
    }

    // ---- 2. Show the user the line ----------------------------------------------------------
    // line() and column() are 1-based. A few lines of code turn them into a
    // compiler-style message.
    let (_, text) = cases[1];
    let err = serde_json::from_str::<Server>(text).unwrap_err();
    println!(
        "\n2. pointing at it:\n{}",
        point_at(text, err.line(), err.column())
    );

    // ---- 3. The toml crate draws the picture for you ------------------------------------------
    let toml_text = "host = \"nas\"\nport = \"80\"\n";
    let err = toml::from_str::<Server>(toml_text).unwrap_err();
    println!("\n3. TOML's Display:\n{err}");
    // The parts, for your own formatting: a byte range and the bare message.
    println!(
        "   span() = {:?}, message() = {:?}",
        err.span(),
        err.message()
    );

    // ---- 4. Internally tagged and untagged enums lose the position -------------------------------
    // To find the tag, Serde first buffers the whole object, then reads the
    // buffered copy, which has no line numbers.
    let err =
        serde_json::from_str::<Job>("{\n  \"kind\": \"Backup\",\n  \"folder\": 7\n}").unwrap_err();
    println!(
        "\n4. internally tagged: {err}  (line {} column {})",
        err.line(),
        err.column()
    );
}

/// A compiler-style snippet: the line, and a caret under the column.
fn point_at(text: &str, line: usize, column: usize) -> String {
    let source = text.lines().nth(line - 1).unwrap_or("");
    format!(
        "   |\n{line:>2} | {source}\n   | {caret:>column$}",
        caret = "^"
    )
}
