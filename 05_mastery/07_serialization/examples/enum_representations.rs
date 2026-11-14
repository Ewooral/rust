//! The four ways Serde can write an enum: externally, internally and
//! adjacently tagged, and untagged.
//! Run: cargo run -p serialization --example enum_representations
//!
//! The same three variants, four times, so the JSON can be compared.

use serde::{Deserialize, Serialize};

// ---- 1. Externally tagged (the default) --------------------------------------------
// {"Circle":{"radius":1.5}}   The variant name WRAPS the data.
#[derive(Debug, Serialize, Deserialize)]
enum External {
    Circle { radius: f64 },
    Square { side: f64 },
    Dot,
}

// ---- 2. Internally tagged: tag = "..." ---------------------------------------------
// {"shape":"Circle","radius":1.5}   The variant name is a field INSIDE.
// Common in web APIs ("type": "...") and config files.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "shape")]
enum Internal {
    Circle {
        radius: f64,
    },
    Square {
        side: f64,
    },
    Dot,
    // `other`: any unknown tag becomes this variant instead of an error.
    #[serde(other)]
    Unknown,
}
// Tuple variants can't be internally tagged (no field names to sit next to):
//     #[serde(tag = "shape")] enum Bad { Line(f64, f64) }
// error: #[serde(tag = "...")] cannot be used with tuple variants   (no error code:
// it's the derive macro speaking, not the compiler)

// ---- 3. Adjacently tagged: tag = "...", content = "..." ----------------------------
// {"shape":"Circle","data":{"radius":1.5}}   Tag and data side by side.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "shape", content = "data")]
enum Adjacent {
    Circle { radius: f64 },
    Square { side: f64 },
    Dot,
}

// ---- 4. Untagged ---------------------------------------------------------------------
// {"radius":1.5}   No name at all. Reading tries each variant IN ORDER and
// keeps the first that fits, so the order of the variants matters.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Untagged {
    Circle { radius: f64 },
    Square { side: f64 },
    Dot,
}

// Untagged is handy for "a number OR a string" fields.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)] // only printed with {:?}
enum IdOrName {
    Id(u64),
    Name(String),
}

fn main() -> Result<(), serde_json::Error> {
    println!("               Circle                                     Dot");
    println!(
        "external:      {:<42} {}",
        serde_json::to_string(&External::Circle { radius: 1.5 })?,
        serde_json::to_string(&External::Dot)?
    );
    println!(
        "internal:      {:<42} {}",
        serde_json::to_string(&Internal::Circle { radius: 1.5 })?,
        serde_json::to_string(&Internal::Dot)?
    );
    println!(
        "adjacent:      {:<42} {}",
        serde_json::to_string(&Adjacent::Circle { radius: 1.5 })?,
        serde_json::to_string(&Adjacent::Dot)?
    );
    println!(
        "untagged:      {:<42} {}",
        serde_json::to_string(&Untagged::Circle { radius: 1.5 })?,
        serde_json::to_string(&Untagged::Dot)?
    );

    println!("\nReading back:");
    let shape: Internal = serde_json::from_str(r#"{"side": 2.0, "shape": "Square"}"#)?;
    println!("  internal, tag last:  {shape:?}   (the tag can be anywhere)");
    let shape: Internal = serde_json::from_str(r#"{"shape": "Hexagon", "sides": 6}"#)?;
    println!("  internal, unknown:   {shape:?}   (thanks to #[serde(other)])");
    let shape: Untagged = serde_json::from_str(r#"{"side": 2.0}"#)?;
    println!("  untagged:            {shape:?}   (Circle didn't fit, Square did)");

    for input in ["7", r#""seven""#] {
        let value: IdOrName = serde_json::from_str(input)?;
        println!("  IdOrName from {input:<8} {value:?}");
    }

    println!("\nWhen nothing fits:");
    let err = serde_json::from_str::<External>(r#"{"Triangle": {}}"#).unwrap_err();
    println!("  external: {err}");
    let err = serde_json::from_str::<Adjacent>(r#"{"data": {"radius": 1.0}}"#).unwrap_err();
    println!("  adjacent: {err}");
    // Untagged errors can't say WHICH variant was closest, or where: Serde
    // buffers the input first, so the position is lost (line 0, column 0).
    let err = serde_json::from_str::<Untagged>(r#"{"radius": "big"}"#).unwrap_err();
    println!(
        "  untagged: {err}  (line {}, column {})",
        err.line(),
        err.column()
    );
    Ok(())
}
