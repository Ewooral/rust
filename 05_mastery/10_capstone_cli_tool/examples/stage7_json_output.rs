//! Stage 7: output for programs. `#[derive(Serialize)]` on the Summary is
//! all `--json` needs; a script can then read exactly the field it wants.
//! Run: cargo run -p capstone_cli_tool --example stage7_json_output

use capstone_cli_tool::{Level, Summary, summarize_text, write_json};

const LOG: &str = "\
2026-05-01T09:00:00Z INFO start
2026-05-01T09:00:01Z ERROR payment 991 failed
2026-05-01T09:00:02Z ERROR payment 17 failed
2026-05-01T09:00:03Z WARN disk 91% full
";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let summary = summarize_text(LOG, Level::Warn, 5);

    // 1. Compact JSON: one line, the smallest output. Good for logs and pipes.
    println!("compact:\n{}\n", serde_json::to_string(&summary)?);

    // 2. Pretty JSON, as `logsum --json` prints it (via write_json).
    println!("pretty:");
    write_json(&mut std::io::stdout().lock(), &summary)?;

    // 3. What a consumer does: parse it back. Untyped, like `jq .levels.error`...
    let json = serde_json::to_string(&summary)?;
    let value: serde_json::Value = serde_json::from_str(&json)?;
    println!(
        "\nvalue[\"levels\"][\"error\"] = {}",
        value["levels"]["error"]
    );
    println!(
        "value[\"top\"][0][\"pattern\"] = {}",
        value["top"][0]["pattern"]
    );

    // ...or typed, back into the same struct (Deserialize is derived too).
    let back: Summary = serde_json::from_str(&json)?;
    println!("round trip equal: {}", back == summary);

    // JSON output is a PROMISE: scripts depend on the field names. Renaming
    // a field is a breaking change (semver major), like changing a pub fn.
    Ok(())
}
