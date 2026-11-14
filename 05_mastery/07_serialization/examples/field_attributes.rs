//! Field and container attributes: change the shape of the JSON, not your Rust.
//! Run: cargo run -p serialization --example field_attributes
//!
//! rename_all, rename, alias, default, skip, skip_serializing_if, flatten,
//! deny_unknown_fields.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// A web API speaks camelCase; Rust speaks snake_case. One container attribute
// renames every field. Other choices: "lowercase", "UPPERCASE", "PascalCase", "snake_case",
// "SCREAMING_SNAKE_CASE", "kebab-case", "SCREAMING-KEBAB-CASE".
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Profile {
    user_id: u64, // "userId"
    display_name: String,

    // rename: one field, any name, even a Rust keyword.
    #[serde(rename = "type")]
    kind: String,

    // alias: ALSO accept another name when reading. Writing uses the main one.
    #[serde(alias = "e-mail", alias = "mail")]
    email: String,

    // default: a missing field uses Default::default() (false, 0, "", empty Vec)...
    #[serde(default)]
    verified: bool,

    // ...or a function of your own.
    #[serde(default = "default_theme")]
    theme: String,

    // skip: never written, never read; filled with Default when reading.
    #[serde(skip)]
    login_count_this_session: u32,

    // skip_serializing_if: leave it out when there's nothing to say.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    nickname: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    badges: Vec<String>,

    // flatten: the fields of `address` sit at THIS level in the JSON.
    #[serde(flatten)]
    address: Address,

    // flatten into a map: catches every field nobody else claimed.
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

fn default_theme() -> String {
    "light".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Address {
    city: String,
    post_code: String,
}

// deny_unknown_fields: a field the struct doesn't know is an ERROR, not ignored.
// Great for config files, where an unknown field is usually a typo.
// (The Serde docs say not to combine it with flatten.)
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct Strict {
    colour: String,
}

// Container-level default: missing fields come from Settings::default().
#[derive(Debug, Deserialize)]
#[serde(default)]
#[allow(dead_code)]
struct Settings {
    volume: u8,
    muted: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            volume: 7,
            muted: false,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Reading: note "mail" (an alias), no "verified"/"theme"/"nickname"/"badges"
    // (defaults), and "plan", which no field claims (it lands in `extra`).
    let input = r#"{
        "userId": 42,
        "displayName": "Ama",
        "type": "admin",
        "mail": "ama@example.com",
        "city": "Accra",
        "postCode": "GA-123",
        "plan": "family"
    }"#;
    let mut profile: Profile = serde_json::from_str(input)?;
    profile.login_count_this_session = 3;
    println!("read:\n{profile:#?}\n");

    // Writing: camelCase names, "type", "email" (not the alias), no nickname or
    // badges, no login count, the address flattened, "plan" kept from `extra`.
    println!("written:\n{}\n", serde_json::to_string_pretty(&profile)?);

    // deny_unknown_fields turns a typo into a clear error:
    let typo = serde_json::from_str::<Strict>(r#"{ "color": "red" }"#);
    println!("deny_unknown_fields: {}", typo.unwrap_err());

    // Without it, the same typo is silently ignored, and you get a different
    // error, or worse, a default value and no error at all:
    let quiet = serde_json::from_str::<Settings>(r#"{ "volum": 11 }"#)?;
    println!("container default, typo ignored: {quiet:?}  <- volume is still 7!");
    Ok(())
}
