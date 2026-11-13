//! A SECOND binary crate in the same package.
//!
//! A package may contain at most one library crate but any number of binary
//! crates: each file in src/bin/ becomes its own program, named after the file.
//!
//! Run: cargo run -p packages_crates_modules --bin kitchen

use restaurant::back_of_house::{self, Appetizer, Breakfast};

fn main() {
    println!("kitchen display");
    println!("  today's fruit: {}", Breakfast::summer("any").fruit());
    for starter in [Appetizer::Soup, Appetizer::Salad] {
        println!(
            "  {starter:?} costs {}p",
            restaurant::serving::price_of(&starter)
        );
    }
    println!("  {}", back_of_house::fix_incorrect_order());
    println!("  {}", back_of_house::chef_note());
}
