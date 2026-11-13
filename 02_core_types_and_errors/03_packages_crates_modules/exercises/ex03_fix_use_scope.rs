//! Exercise 3: make it compile (E0433).
//!
//! The `use` is there, so why can't `customer` see `hosting`? Fix it by
//! moving one line. (There is a second fix using `super::`. Try both.)
//!
//! Run: cargo run -p packages_crates_modules --features exercises --bin ex03_fix_use_scope

mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() -> &'static str {
            "added to the waitlist"
        }
    }
}

use crate::front_of_house::hosting;

mod customer {
    pub fn eat_at_restaurant() -> &'static str {
        hosting::add_to_waitlist()
    }
}

fn main() {
    println!("{}", customer::eat_at_restaurant());
}
