//! Exercise 3: make it compile (E0599).
//!
//! A shopping list stores its items as ONE comma-separated string:
//!
//!     {"shop": "market", "items": "yam, plantain, eggs"}
//!
//! The `comma_list` module is a `with` module: Serde calls its `serialize`
//! and `deserialize` functions for the `items` field. The trick inside
//! `deserialize` is to read a plain String first, with `String::deserialize`.
//!
//! But `deserialize` is a method of the `Deserialize` TRAIT, and a trait's
//! methods can only be called when the trait is in scope. The `use` at the
//! top of the file doesn't count: a module starts with an empty scope.
//!
//! Fix the module (one line). Then explain why `serializer.serialize_str`
//! works without importing anything extra.
//!
//! Run: cargo run -p serialization --features exercises --bin ex03_fix_trait_not_in_scope

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct ShoppingList {
    shop: String,
    #[serde(with = "comma_list")]
    items: Vec<String>,
}

mod comma_list {
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(items: &[String], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&items.join(", "))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<String>, D::Error> {
        let text = String::deserialize(deserializer)?;
        Ok(text
            .split(',')
            .map(|item| item.trim().to_string())
            .collect())
    }
}

fn main() {
    let json = r#"{"shop": "market", "items": "yam, plantain, eggs"}"#;
    let list: ShoppingList = serde_json::from_str(json).unwrap();
    println!("{list:?}");
    assert_eq!(list.items, ["yam", "plantain", "eggs"]);
    println!("written back: {}", serde_json::to_string(&list).unwrap());
}
