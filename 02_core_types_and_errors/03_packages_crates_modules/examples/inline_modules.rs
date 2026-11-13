//! Defining modules inline with `mod name { ... }`, and privacy.
//! Run: cargo run -p packages_crates_modules --example inline_modules

// Everything is PRIVATE by default: modules, functions, structs, constants.
// `pub` opens it up to the parent module (and anyone who can see the parent).
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() -> &'static str {
            seat_at_table_later()
        }

        // Private: only callable from inside `hosting` and its children.
        fn seat_at_table_later() -> &'static str {
            "added to the waitlist"
        }
    }

    // A private module: `pub` items inside it are still unreachable from
    // outside `front_of_house`, because you cannot get past this door.
    mod serving {
        pub fn take_order() {}
    }

    pub fn open_up() -> &'static str {
        serving::take_order(); // fine: we are inside front_of_house
        "doors open"
    }
}

fn main() {
    // `front_of_house` itself is private, but it is our CHILD and a sibling
    // item in the same module, so we may name it.
    println!("{}", front_of_house::open_up());
    println!("{}", front_of_house::hosting::add_to_waitlist());

    // front_of_house::hosting::seat_at_table_later(); // E0603: function `seat_at_table_later` is private
    // front_of_house::serving::take_order();          // E0603: module `serving` is private
}
