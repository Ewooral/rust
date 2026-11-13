//! Mini project for Book chapter 7: the BINARY crate of this package.
//!
//! This file is a separate crate from src/lib.rs. It uses the `restaurant`
//! library exactly as any outside crate would: only `pub` items are visible,
//! through the paths the library chose to publish.
//!
//! Run: cargo run -p packages_crates_modules
//! Test: cargo test -p packages_crates_modules

use restaurant::back_of_house::{Appetizer, Breakfast};
use restaurant::{hosting, serving}; // thanks to `pub use` in lib.rs

fn main() {
    let mut waitlist = hosting::Waitlist::new();
    for guest in ["Ama", "Kofi", "Esi"] {
        hosting::add_to_waitlist(&mut waitlist, guest);
        println!("{guest} joined the waitlist ({} waiting)", waitlist.len());
    }

    while let Some(seated) = serving::serve_next(&mut waitlist) {
        println!("{seated}");
    }

    let breakfast = Breakfast::summer("Rye");
    println!("{} toast with {}", breakfast.toast, breakfast.fruit());

    let total = serving::take_order(&[Appetizer::Soup, Appetizer::Salad, Appetizer::Soup]);
    println!("{}", serving::take_payment(total));

    println!("--- the Book's eat_at_restaurant ---");
    for line in restaurant::eat_at_restaurant() {
        println!("{line}");
    }

    // None of these compile from here. Uncomment one to see why:
    // restaurant::front_of_house::hosting::Waitlist::new(); // E0603: module `front_of_house` is private
    // restaurant::kitchen_secret();                          // E0603: function `kitchen_secret` is private (pub(crate))
    // serving::tip(100);                                     // E0603: function `tip` is private (pub(super))
}
