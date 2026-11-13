//! The `restaurant` LIBRARY crate. This file is its crate root.
//!
//! The package is called `packages_crates_modules` (see Cargo.toml), but
//! `[lib] name = "restaurant"` gives the library crate its own name, so other
//! crates write `use restaurant::...`.
//!
//! The module tree, and the file each module lives in:
//!
//! ```text
//! crate (restaurant)                 src/lib.rs
//! ├── front_of_house  (private)      src/front_of_house.rs
//! │   ├── hosting     (pub)          src/front_of_house/hosting.rs
//! │   └── serving     (pub)          src/front_of_house/serving.rs
//! └── back_of_house   (pub)          src/back_of_house/mod.rs   (older style)
//! ```

// `mod name;` (with a semicolon) means "the body of this module is in a file".
// The compiler looks for src/front_of_house.rs or src/front_of_house/mod.rs.
pub mod back_of_house;
mod front_of_house;

// Re-exports: `front_of_house` stays private, but these two modules are
// published at the top of the crate. Users write `restaurant::hosting`, and we
// are free to reorganise the inside later without breaking them.
pub use crate::front_of_house::hosting;
pub use crate::front_of_house::serving;

use back_of_house::{Appetizer, Breakfast};

/// Called by `back_of_house::fix_incorrect_order` through `super::`.
/// Private, yet visible to child modules: children can see their ancestors.
fn deliver_order() -> &'static str {
    "order delivered"
}

/// The Book's `eat_at_restaurant`, returning what happened so it can be tested.
pub fn eat_at_restaurant() -> Vec<String> {
    let mut log = Vec::new();

    // Absolute path: starts from the crate root with `crate`.
    let mut waitlist = crate::front_of_house::hosting::Waitlist::new();
    // Relative path: starts from the current module (here, the crate root).
    front_of_house::hosting::add_to_waitlist(&mut waitlist, "Ama");
    // Through the `use`/`pub use` shortcut.
    hosting::add_to_waitlist(&mut waitlist, "Kofi");
    log.push(format!("waiting: {}", waitlist.len()));

    // Order a breakfast in summer with Rye toast.
    let mut meal = Breakfast::summer("Rye");
    // Change our mind about the toast: the field is `pub`.
    meal.toast = String::from("Wheat");
    log.push(format!("I'd like {} toast please", meal.toast));
    // meal.seasonal_fruit = String::from("blueberries"); // E0616: field is private
    log.push(format!("fruit of the season: {}", meal.fruit()));

    // Enum variants are public as soon as the enum is.
    let starters = [Appetizer::Soup, Appetizer::Salad];
    log.push(format!("starters: {starters:?}"));

    log.push(back_of_house::fix_incorrect_order());
    log
}

/// `pub(crate)`: visible everywhere inside this library, invisible outside it.
/// `src/main.rs` is a DIFFERENT crate, so it cannot call this (E0603).
pub(crate) fn kitchen_secret() -> &'static str {
    "the soup is from yesterday"
}

#[cfg(test)]
mod tests {
    // `tests` is a child of the crate root, so `super::*` brings in every item
    // of the root, private ones included.
    use super::*;

    #[test]
    fn eat_at_restaurant_runs_through_the_whole_meal() {
        let log = eat_at_restaurant();
        assert_eq!(log[0], "waiting: 2");
        assert_eq!(log[1], "I'd like Wheat toast please");
        assert_eq!(log[2], "fruit of the season: peaches");
        assert_eq!(log.last().unwrap(), "fixed: order delivered");
    }

    #[test]
    fn private_items_are_visible_inside_the_crate() {
        assert_eq!(deliver_order(), "order delivered");
        assert_eq!(kitchen_secret(), "the soup is from yesterday");
    }
}
