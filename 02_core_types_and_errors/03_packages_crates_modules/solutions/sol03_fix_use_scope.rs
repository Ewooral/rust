//! Solution to exercise 3: make it compile (E0433).
//!
//! The `use` is there, so why can't `customer` see `hosting`? Fix it by
//! moving one line. (There is a second fix using `super::`. Try both.)
//!
//! Run: cargo run -p packages_crates_modules --features solutions --bin sol03_fix_use_scope

mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() -> &'static str {
            "added to the waitlist"
        }
    }
}

mod customer {
    // A `use` only applies to the module it is written in.
    // (Or, without this line: `super::hosting::add_to_waitlist()`.)
    use crate::front_of_house::hosting;

    pub fn eat_at_restaurant() -> &'static str {
        hosting::add_to_waitlist()
    }
}

fn main() {
    println!("{}", customer::eat_at_restaurant());
}
