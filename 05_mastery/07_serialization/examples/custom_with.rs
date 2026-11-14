//! Customising ONE field: serialize_with, deserialize_with, with = "module",
//! and validating newtypes with try_from / into.
//! Run: cargo run -p serialization --example custom_with

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Lamp {
    name: String,

    // serialize_with: a function used only when WRITING this field.
    // (Reading still uses f32's normal Deserialize, so it reads 0.75.)
    #[serde(serialize_with = "as_percent")]
    brightness: f32,

    // deserialize_with: a function used only when READING this field.
    // "kitchen, warm , dimmable" becomes ["kitchen", "warm", "dimmable"].
    #[serde(deserialize_with = "comma_list")]
    tags: Vec<String>,

    // with: a module with BOTH functions, `serialize` and `deserialize`.
    #[serde(with = "hex_colour")]
    colour: [u8; 3],

    // try_from on the TYPE: checked every time a Room is read, anywhere.
    room: Room,
}

/// `fn(&FieldType, S) -> Result<S::Ok, S::Error>`: the shape every
/// serialize_with function has.
fn as_percent<S: Serializer>(value: &f32, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&format!("{:.0}%", value * 100.0))
}

/// `fn(D) -> Result<FieldType, D::Error>`: the shape of a deserialize_with.
/// The trick: deserialize something Serde already understands (a String),
/// then convert it. `String::deserialize` needs the `Deserialize` TRAIT in scope.
fn comma_list<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<String>, D::Error> {
    let text = String::deserialize(deserializer)?;
    Ok(text
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(String::from)
        .collect())
}

/// A `with` module: `[255, 136, 0]` <-> `"#ff8800"`.
mod hex_colour {
    // A module doesn't see the `use` lines of its parent: import again.
    use serde::de::Error as _; // brings `D::Error::custom` into scope
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(rgb: &[u8; 3], serializer: S) -> Result<S::Ok, S::Error> {
        let [r, g, b] = rgb;
        serializer.serialize_str(&format!("#{r:02x}{g:02x}{b:02x}"))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<[u8; 3], D::Error> {
        let text = String::deserialize(deserializer)?;
        let hex = text.strip_prefix('#').unwrap_or(&text);
        if hex.len() != 6 || !hex.is_ascii() {
            // D::Error::custom turns any message into the format's own error
            // type, so it gets the line and column like any other error.
            return Err(D::Error::custom(format!(
                "expected a colour like \"#ff8800\", found {text:?}"
            )));
        }
        let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).map_err(D::Error::custom);
        Ok([byte(0)?, byte(2)?, byte(4)?])
    }
}

/// try_from = "String": read a String, then call Room::try_from(String).
/// into = "String": write by converting a clone into a String.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
struct Room(String);

impl TryFrom<String> for Room {
    type Error = String; // anything that implements Display

    fn try_from(name: String) -> Result<Room, String> {
        const ROOMS: [&str; 4] = ["kitchen", "hall", "bedroom", "garden"];
        if ROOMS.contains(&name.as_str()) {
            Ok(Room(name))
        } else {
            Err(format!("no room called {name:?}; try one of {ROOMS:?}"))
        }
    }
}

impl From<Room> for String {
    fn from(room: Room) -> String {
        room.0
    }
}

fn main() -> Result<(), serde_json::Error> {
    let lamp = Lamp {
        name: "Ceiling".to_string(),
        brightness: 0.75,
        tags: vec!["warm".to_string(), "dimmable".to_string()],
        colour: [255, 136, 0],
        room: Room("kitchen".to_string()),
    };
    println!("written: {}", serde_json::to_string(&lamp)?);

    let input = r##"{
        "name": "Desk",
        "brightness": 0.5,
        "tags": "office, reading ,  ",
        "colour": "#00ccff",
        "room": "bedroom"
    }"##;
    let desk: Lamp = serde_json::from_str(input)?;
    println!("read:    {desk:?}");

    println!("\nErrors from our own code still get a line and column:");
    let bad_colour = input.replace("#00ccff", "teal");
    println!(
        "  {}",
        serde_json::from_str::<Lamp>(&bad_colour).unwrap_err()
    );
    let bad_room = input.replace("bedroom", "attic");
    println!("  {}", serde_json::from_str::<Lamp>(&bad_room).unwrap_err());
    Ok(())
}
