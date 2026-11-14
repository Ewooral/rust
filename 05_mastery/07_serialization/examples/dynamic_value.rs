//! serde_json::Value: JSON whose shape you don't know in advance.
//! Run: cargo run -p serialization --example dynamic_value
//!
//! 1. the json! macro   2. reading: index, get, pointer, as_*   3. changing it
//! 4. Value <-> your types   5. any self-describing format can become a Value

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Serialize, Deserialize)]
struct Weather {
    city: String,
    celsius: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ---- 1. Build a Value with json! (any Rust expression can go inside) ------------------
    let rain_chance = 0.4;
    let mut report = json!({
        "zone": "West Africa",
        "cities": [
            { "city": "Accra", "celsius": 31.5, "alerts": [] },
            { "city": "Kumasi", "celsius": 28.0, "alerts": ["storm"] }
        ],
        "rain": rain_chance,
        "updated": null
    });
    // Value is an enum: Null, Bool, Number, String, Array, Object.
    println!("1. {report}\n");

    // ---- 2. Read it -----------------------------------------------------------------------
    // Indexing never panics: a missing key or a wrong type gives Value::Null.
    println!(
        "2. report[\"cities\"][1][\"city\"]  = {}",
        report["cities"][1]["city"]
    );
    println!(
        "   report[\"nope\"][\"deeper\"]    = {}",
        report["nope"]["deeper"]
    );
    // .get() returns an Option instead, so you can tell "missing" from "null".
    println!(
        "   report.get(\"updated\")       = {:?}",
        report.get("updated")
    );
    println!(
        "   report.get(\"missing\")       = {:?}",
        report.get("missing")
    );
    // A JSON Pointer (RFC 6901) walks a path in one go.
    println!(
        "   pointer(\"/cities/1/alerts/0\") = {:?}",
        report.pointer("/cities/1/alerts/0")
    );
    // as_str, as_f64, as_array, ... return Option: None if it's another type.
    let hottest = report["cities"]
        .as_array()
        .into_iter()
        .flatten()
        .max_by(|a, b| {
            a["celsius"]
                .as_f64()
                .partial_cmp(&b["celsius"].as_f64())
                .unwrap()
        })
        .and_then(|city| city["city"].as_str());
    println!("   hottest city: {hottest:?}\n");

    // ---- 3. Change it ---------------------------------------------------------------------
    report["updated"] = json!("2026-11-14T09:00Z");
    report["cities"][0]["alerts"]
        .as_array_mut()
        .ok_or("alerts should be a list")?
        .push(json!("heat"));
    if let Some(object) = report.as_object_mut() {
        object.remove("rain");
        object.insert("source".to_string(), json!("lesson"));
    }
    // serde_json's Map keeps keys sorted, unless you enable its
    // `preserve_order` feature (then it keeps the order you inserted).
    println!("3. {}\n", serde_json::to_string_pretty(&report)?);

    // ---- 4. Convert between Value and your own types ----------------------------------------
    // from_value: a Value -> a typed struct (unknown fields like "alerts" are ignored).
    let kumasi: Weather = serde_json::from_value(report["cities"][1].clone())?;
    println!("4. from_value: {kumasi:?}");
    // to_value: any Serialize type -> a Value, to inspect or edit generically.
    let as_value = serde_json::to_value(&kumasi)?;
    println!("   to_value:   {as_value}");
    // Wrong shape: the same errors as from_str, but without a line and column.
    let err = serde_json::from_value::<Weather>(json!({ "city": "Tamale" })).unwrap_err();
    println!("   error:      {err}\n");

    // ---- 5. Values from other formats ---------------------------------------------------------
    // serde_json::Value implements Deserialize, so the toml crate can build one.
    let cargo_toml =
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n\n[dependencies]\nserde = \"1\"\n";
    let manifest: Value = toml::from_str(cargo_toml)?;
    println!("5. a Cargo.toml as JSON: {manifest}");
    println!("   package name: {}", manifest["package"]["name"]);
    Ok(())
}
