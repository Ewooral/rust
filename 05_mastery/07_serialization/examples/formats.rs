//! One struct, three formats: JSON (serde_json), TOML (toml) and binary (postcard).
//! Run: cargo run -p serialization --example formats
//!
//! The struct derives Serialize/Deserialize ONCE. Each format crate brings its
//! own Serializer and Deserializer; the struct never knows which one it meets.

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Recipe {
    name: String,
    serves: u8,
    minutes: u16,
    vegetarian: bool,
    ingredients: Vec<Ingredient>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Ingredient {
    item: String,
    grams: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pancakes = Recipe {
        name: "Pancakes".to_string(),
        serves: 4,
        minutes: 300, // a slow Sunday
        vegetarian: true,
        ingredients: vec![
            Ingredient {
                item: "flour".to_string(),
                grams: 200,
            },
            Ingredient {
                item: "milk".to_string(),
                grams: 300,
            },
        ],
    };

    // ---- JSON: the web's format ----------------------------------------------------
    let json = serde_json::to_string(&pancakes)?;
    println!("JSON ({} bytes):\n{json}\n", json.len());

    // ---- TOML: config files (Cargo.toml!) ------------------------------------------
    // A Vec of structs becomes an "array of tables": [[ingredients]].
    let toml_text = toml::to_string(&pancakes)?;
    println!("TOML ({} bytes):\n{toml_text}", toml_text.len());

    // ---- postcard: compact binary --------------------------------------------------
    // No field names, no quotes: just the values, in field order. Integers are
    // "varints": small numbers take fewer bytes (300 takes 2, 4 takes 1).
    let bytes = postcard::to_stdvec(&pancakes)?;
    println!("postcard ({} bytes):\n{bytes:?}\n", bytes.len());

    // All three read back to the same value:
    assert_eq!(serde_json::from_str::<Recipe>(&json)?, pancakes);
    assert_eq!(toml::from_str::<Recipe>(&toml_text)?, pancakes);
    assert_eq!(postcard::from_bytes::<Recipe>(&bytes)?, pancakes);
    println!("all three round-trip to the same Recipe\n");

    // ---- What each format can't do -------------------------------------------------
    // TOML's top level must be a table (a struct or a map), and TOML has no null.
    println!("TOML, a bare list:   {:?}", toml::to_string(&vec![1, 2, 3]));
    println!(
        "TOML, a null:        {:?}",
        toml::to_string(&serde_json::json!({ "a": null }))
    );

    // postcard doesn't describe itself: the bytes don't say "this is a string",
    // so it can't fill a type that asks "what have you got?" (serde_json::Value).
    let as_value = postcard::from_bytes::<serde_json::Value>(&bytes);
    println!("postcard -> Value:   {as_value:?}");

    // And it trusts that the reader uses the SAME struct, in the same order.
    // Cut the bytes short and it notices; give it a different struct and it may not.
    println!(
        "postcard, cut short: {:?}",
        postcard::from_bytes::<Recipe>(&bytes[..10])
    );
    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct Imposter {
        title: String,
        level: u8,
    }
    println!(
        "postcard, wrong struct: {:?}  <- no error!",
        postcard::from_bytes::<Imposter>(&bytes)
    );
    Ok(())
}
