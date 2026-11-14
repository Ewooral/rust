//! Serde basics: derive `Serialize` and `Deserialize`, then JSON with serde_json.
//! Run: cargo run -p serialization --example derive_and_json
//!
//! 1. derive   2. to_string / to_string_pretty   3. from_str (and the type annotation)
//! 4. to_writer / from_reader / from_slice   5. what std types look like in JSON

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// ---- 1. Derive -------------------------------------------------------------------
// The derive writes `impl Serialize for Pet` and `impl Deserialize for Pet` for
// you. Every field's type must implement the trait too: String, u8, Vec, Option
// and Species (derived below) all do.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Pet {
    name: String,
    species: Species,
    age: u8,
    toys: Vec<String>,
    microchip: Option<u64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Species {
    Cat,
    Dog,
    Rabbit,
}

// Leave a derive off a field's type and the outer derive can't compile:
// struct Collar { colour: String }                 // no derive
// #[derive(Serialize)] struct Pet2 { collar: Collar }
// E0277: the trait bound `Collar: serde::Serialize` is not satisfied

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rex = Pet {
        name: "Rex".to_string(),
        species: Species::Dog,
        age: 3,
        toys: vec!["ball".to_string(), "rope".to_string()],
        microchip: None,
    };

    // ---- 2. Rust value -> JSON text ------------------------------------------------
    let compact = serde_json::to_string(&rex)?;
    println!("to_string:\n{compact}\n");
    let pretty = serde_json::to_string_pretty(&rex)?;
    println!("to_string_pretty:\n{pretty}\n");

    // ---- 3. JSON text -> Rust value ------------------------------------------------
    // The annotation `: Pet` is how serde_json knows WHAT to build. The same
    // text could become a Pet, a serde_json::Value, or a HashMap.
    let back: Pet = serde_json::from_str(&compact)?;
    assert_eq!(back, rex);
    println!("from_str gave back: {back:?}\n");

    // Field order in the input doesn't matter, and whitespace is ignored:
    let mia: Pet = serde_json::from_str(
        r#"{ "age": 7, "species": "Cat", "toys": [], "microchip": 900123, "name": "Mia" }"#,
    )?;
    println!("fields in any order: {mia:?}\n");

    // ---- 4. Readers and writers ----------------------------------------------------
    // to_writer / from_reader work with any io::Write / io::Read: a File, a
    // TcpStream, or a Vec<u8> like here. from_slice reads bytes you already have.
    let mut buffer: Vec<u8> = Vec::new();
    serde_json::to_writer(&mut buffer, &rex)?;
    let from_reader: Pet = serde_json::from_reader(buffer.as_slice())?;
    let from_slice: Pet = serde_json::from_slice(&buffer)?;
    assert_eq!(from_reader, from_slice);
    println!("{} bytes written, read back twice\n", buffer.len());

    // ---- 5. Standard types, out of the box -----------------------------------------
    let mut scores = BTreeMap::new();
    scores.insert("Ama", 12);
    scores.insert("Kofi", 9);
    println!("map:    {}", serde_json::to_string(&scores)?);
    println!("tuple:  {}", serde_json::to_string(&(1, "two", 3.5))?);
    println!("array:  {}", serde_json::to_string(&[1u8, 2, 3])?);
    println!(
        "Option: {} and {}",
        serde_json::to_string(&Some(5))?,
        serde_json::to_string(&None::<i32>)?
    );
    println!("unit:   {}", serde_json::to_string(&())?);
    println!("char:   {}", serde_json::to_string(&'é')?);
    Ok(())
}
