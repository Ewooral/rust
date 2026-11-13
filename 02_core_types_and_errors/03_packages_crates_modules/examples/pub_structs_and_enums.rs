//! `pub` on structs and enums: fields choose separately, variants do not.
//! Run: cargo run -p packages_crates_modules --example pub_structs_and_enums

mod back_of_house {
    #[derive(Debug)]
    pub struct Breakfast {
        pub toast: String,
        seasonal_fruit: String,
    }

    impl Breakfast {
        pub fn summer(toast: &str) -> Breakfast {
            Breakfast {
                toast: String::from(toast),
                seasonal_fruit: String::from("peaches"),
            }
        }

        pub fn fruit(&self) -> &str {
            &self.seasonal_fruit
        }
    }

    #[derive(Debug)]
    pub enum Appetizer {
        Soup,
        Salad,
    }
}

use back_of_house::{Appetizer, Breakfast};

fn main() {
    let mut meal = Breakfast::summer("Rye");
    meal.toast = String::from("Wheat"); // pub field: fine
    println!(
        "I'd like {} toast please, with {}",
        meal.toast,
        meal.fruit()
    );

    // meal.seasonal_fruit = String::from("blueberries");
    // ^ E0616: field `seasonal_fruit` of struct `Breakfast` is private

    // let custom = Breakfast { toast: String::from("Rye"), seasonal_fruit: String::from("kiwi") };
    // ^ E0451: field `seasonal_fruit` of struct `Breakfast` is private
    //   (one private field means outsiders can't build it: use the constructor)

    // A pub enum's variants are all pub. An enum with private variants would
    // be useless: you could not match on it.
    let order1 = Appetizer::Soup;
    let order2 = Appetizer::Salad;
    println!("{order1:?} and {order2:?}");
}
