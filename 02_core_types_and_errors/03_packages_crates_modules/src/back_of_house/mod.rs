//! `crate::back_of_house`, written in the OLDER file layout:
//! `back_of_house/mod.rs` instead of `back_of_house.rs`.
//! Both still work; don't mix the two styles for the same module.

/// A public struct does NOT make its fields public. Each field chooses.
#[derive(Debug)]
pub struct Breakfast {
    pub toast: String,
    seasonal_fruit: String, // the kitchen decides, not the customer
}

impl Breakfast {
    /// Because `seasonal_fruit` is private, code outside this module cannot
    /// write `Breakfast { .. }` at all. It must go through a constructor.
    pub fn summer(toast: &str) -> Breakfast {
        Breakfast {
            toast: String::from(toast),
            seasonal_fruit: String::from("peaches"),
        }
    }

    /// A getter: read-only access to a private field.
    pub fn fruit(&self) -> &str {
        &self.seasonal_fruit
    }
}

/// A public enum DOES make all its variants public.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Appetizer {
    Soup,
    Salad,
}

pub fn fix_incorrect_order() -> String {
    cook_order();
    // `super` = the parent module, here the crate root.
    format!("fixed: {}", super::deliver_order())
}

fn cook_order() {}

/// Uses a `pub(crate)` item from the crate root: allowed, same crate.
pub fn chef_note() -> String {
    format!("chef says: {}", crate::kitchen_secret())
}
