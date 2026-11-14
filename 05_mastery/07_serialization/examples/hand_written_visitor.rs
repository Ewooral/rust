//! Implementing Serialize and Deserialize BY HAND, with a Visitor.
//! Run: cargo run -p serialization --example hand_written_visitor
//!
//! `Rgb` accepts three shapes when reading:
//!   "#ff8800"                    a string        -> visit_str
//!   [255, 136, 0]                a list          -> visit_seq
//!   {"r": 255, "g": 136, "b": 0} a map           -> visit_map
//! and always writes the string form.

use std::fmt;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

// ---- Serialize: tell the Serializer what we are -------------------------------------
// Serde's data model has 29 types (bool, i8..u128, f32, f64, char, str, bytes,
// option, unit, seq, tuple, map, struct, enum variants...). We say "I am a str".
impl Serialize for Rgb {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let Rgb { r, g, b } = self;
        serializer.serialize_str(&format!("#{r:02x}{g:02x}{b:02x}"))
    }
}

// ---- Deserialize: hand the Deserializer a Visitor ------------------------------------
impl<'de> Deserialize<'de> for Rgb {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Rgb, D::Error> {
        // deserialize_any: "look at the input and call the matching visit_*".
        // A struct with one shape would give a HINT instead, e.g.
        // deserializer.deserialize_str(RgbVisitor), which every format supports.
        deserializer.deserialize_any(RgbVisitor)
    }
}

struct RgbVisitor;

impl<'de> Visitor<'de> for RgbVisitor {
    type Value = Rgb;

    // Finishes the sentence "invalid type: ..., expected ___".
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a colour: \"#rrggbb\", [r, g, b] or {\"r\", \"g\", \"b\"}")
    }

    fn visit_str<E: de::Error>(self, text: &str) -> Result<Rgb, E> {
        let hex = text.strip_prefix('#').unwrap_or(text);
        let channel = |i: usize| {
            hex.get(i..i + 2)
                .and_then(|h| u8::from_str_radix(h, 16).ok())
        };
        match (hex.len(), channel(0), channel(2), channel(4)) {
            (6, Some(r), Some(g), Some(b)) => Ok(Rgb { r, g, b }),
            // invalid_value builds "invalid value: string \"...\", expected <expecting>"
            _ => Err(E::invalid_value(de::Unexpected::Str(text), &self)),
        }
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Rgb, A::Error> {
        // next_element::<u8>() deserializes the next item as a u8, so a 300
        // in the list fails with u8's own error message.
        let r = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let g = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;
        let b = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(2, &self))?;
        if seq.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_length(4, &self));
        }
        Ok(Rgb { r, g, b })
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Rgb, A::Error> {
        let (mut r, mut g, mut b) = (None, None, None);
        while let Some(key) = map.next_key::<String>()? {
            let slot = match key.as_str() {
                "r" => &mut r,
                "g" => &mut g,
                "b" => &mut b,
                other => return Err(de::Error::unknown_field(other, &["r", "g", "b"])),
            };
            if slot.is_some() {
                return Err(de::Error::custom(format!("duplicate field `{key}`")));
            }
            *slot = Some(map.next_value()?);
        }
        Ok(Rgb {
            r: r.ok_or_else(|| de::Error::missing_field("r"))?,
            g: g.ok_or_else(|| de::Error::missing_field("g"))?,
            b: b.ok_or_else(|| de::Error::missing_field("b"))?,
        })
    }
}

fn main() {
    let orange = Rgb {
        r: 255,
        g: 136,
        b: 0,
    };
    println!("written: {}", serde_json::to_string(&orange).unwrap());

    println!("\nReading (each line calls a different visit_ method):");
    for input in [
        r##""#ff8800""##,
        "[255, 136, 0]",
        r#"{"b": 0, "g": 136, "r": 255}"#,
        // and the errors our Visitor produces:
        "true",
        r##""#12345""##,
        "[255, 136]",
        "[255, 136, 0, 9]",
        "[300, 0, 0]",
        r#"{"r": 1, "g": 2}"#,
        r#"{"r": 1, "g": 2, "b": 3, "a": 4}"#,
    ] {
        match serde_json::from_str::<Rgb>(input) {
            Ok(colour) => println!("  {input:<34} -> {colour:?}"),
            Err(err) => println!("  {input:<34} -> error: {err}"),
        }
    }

    // TOML hands us the same visit_* calls: the Visitor doesn't know the format.
    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct Theme {
        text: Rgb,
        background: Rgb,
    }
    let theme: Theme =
        toml::from_str("text = \"#222222\"\nbackground = [250, 250, 245]\n").unwrap();
    println!("\nfrom TOML: {theme:?}");

    // deserialize_any needs a format that describes itself. postcard doesn't:
    let bytes = postcard::to_stdvec(&orange).unwrap();
    println!(
        "postcard bytes {bytes:?} -> {:?}",
        postcard::from_bytes::<Rgb>(&bytes)
    );
}
