//! `use`: shortcuts to paths, and the idioms for using them well.
//! Run: cargo run -p packages_crates_modules --example use_keyword

// Idiom 1: for FUNCTIONS, bring the parent module into scope and call
// `hosting::add_to_waitlist()`. The module name shows it is not local.
// Idiom 2: for structs, enums and other types, bring in the full path:
// `HashMap::new()`, not `collections::HashMap::new()`.
use std::collections::HashMap;

// `as` renames, for when two items share a name.
use std::fmt::Result;
use std::io::Result as IoResult;

// Nested paths: one line instead of two.
// `use std::io::{self, Write};` brings in both `std::io` and `std::io::Write`.
use std::cmp::{Ordering, max};

mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() -> &'static str {
            "waitlisted"
        }
    }
}

use crate::front_of_house::hosting;

mod customer {
    // A `use` only applies in the scope (module) where it is written.
    // The `use crate::front_of_house::hosting` above does NOT reach in here:
    // pub fn eat_at_restaurant() -> &'static str { hosting::add_to_waitlist() }
    // ^ E0433: cannot find module or crate `hosting` in this scope

    // Fix 1: a `use` inside this module. Fix 2: `super::hosting::...`.
    use crate::front_of_house::hosting;

    pub fn eat_at_restaurant() -> &'static str {
        hosting::add_to_waitlist()
    }
}

fn format_ok() -> Result {
    Ok(())
}

fn io_ok() -> IoResult<()> {
    Ok(())
}

fn main() {
    println!("{}", hosting::add_to_waitlist());
    println!("{}", customer::eat_at_restaurant());

    let mut scores = HashMap::new();
    scores.insert("Ama", 3);
    println!("{scores:?}");

    println!("fmt::Result {:?}, io::Result {:?}", format_ok(), io_ok());
    println!("max {}, cmp {:?}", max(3, 7), 3.cmp(&7) == Ordering::Less);

    // The glob operator brings in EVERYTHING public from a path. Handy in
    // tests (`use super::*;`) and preludes; elsewhere it hides where names
    // come from.
    {
        use std::collections::*;
        let set: HashSet<i32> = [1, 2, 2].into_iter().collect();
        println!("glob: {set:?}");
    }
}
