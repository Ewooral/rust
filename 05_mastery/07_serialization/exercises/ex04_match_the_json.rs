//! Exercise 4: match the JSON (pass 3 tests) using only serde attributes.
//!
//! A shop's web API sends orders like this:
//!
//!     {
//!       "orderId": 7,
//!       "customerName": "Ama",
//!       "type": "pickup",
//!       "items": [ { "sku": "YAM-1", "qty": 2 } ],
//!       "payment": { "method": "card", "last4": "4242" }
//!     }
//!
//! Don't change any field names, types or test code: add `#[serde(...)]`
//! attributes to the three types until the tests pass. You'll need:
//!   - camelCase names for the whole struct, and "type" for `kind`
//!   - old clients send "customer" instead of "customerName": accept both
//!   - `notes` is left out of the JSON when it is None
//!   - a missing "items" means an empty list; a missing "qty" means 1
//!     (the function `one` is there for you)
//!   - a misspelt field on an Order is an error, not silently ignored
//!   - `Payment` is internally tagged with "method", in lowercase
//!
//! Test: cargo test -p serialization --features exercises --bin ex04_match_the_json

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Order {
    order_id: u64,
    customer_name: String,
    kind: String,
    notes: Option<String>,
    items: Vec<Item>,
    payment: Payment,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Item {
    sku: String,
    qty: u32,
}

#[allow(dead_code)]
fn one() -> u32 {
    1
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Payment {
    Card { last4: String },
    Cash,
}

fn main() {
    let order = Order {
        order_id: 7,
        customer_name: "Ama".to_string(),
        kind: "pickup".to_string(),
        notes: None,
        items: vec![Item {
            sku: "YAM-1".to_string(),
            qty: 2,
        }],
        payment: Payment::Card {
            last4: "4242".to_string(),
        },
    };
    println!("{}", serde_json::to_string_pretty(&order).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_the_api_shape() {
        let order = Order {
            order_id: 7,
            customer_name: "Ama".to_string(),
            kind: "pickup".to_string(),
            notes: None,
            items: vec![Item {
                sku: "YAM-1".to_string(),
                qty: 2,
            }],
            payment: Payment::Card {
                last4: "4242".to_string(),
            },
        };
        assert_eq!(
            serde_json::to_string(&order).unwrap(),
            r#"{"orderId":7,"customerName":"Ama","type":"pickup","items":[{"sku":"YAM-1","qty":2}],"payment":{"method":"card","last4":"4242"}}"#
        );
    }

    #[test]
    fn reads_old_clients_and_missing_fields() {
        let json = r#"{
            "orderId": 8,
            "customer": "Kofi",
            "type": "delivery",
            "notes": "ring twice",
            "payment": { "method": "cash" }
        }"#;
        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.customer_name, "Kofi");
        assert_eq!(order.notes.as_deref(), Some("ring twice"));
        assert!(order.items.is_empty());
        assert_eq!(order.payment, Payment::Cash);

        let json = r#"{"orderId":9,"customerName":"Esi","type":"pickup",
                       "items":[{"sku":"EGG-6"}],"payment":{"method":"cash"}}"#;
        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.items[0].qty, 1);
    }

    #[test]
    fn typos_are_errors() {
        let json =
            r#"{"orderId":9,"custmerName":"Esi","type":"pickup","payment":{"method":"cash"}}"#;
        let err = serde_json::from_str::<Order>(json).unwrap_err();
        assert!(
            err.to_string().starts_with("unknown field `custmerName`"),
            "{err}"
        );
    }
}
