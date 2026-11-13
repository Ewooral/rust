//! Paths: absolute (`crate::`), relative, and `super::`.
//! Run: cargo run -p packages_crates_modules --example paths

fn deliver_order() -> &'static str {
    "delivered"
}

mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() -> &'static str {
            "waitlisted"
        }
    }
}

mod back_of_house {
    pub fn fix_incorrect_order() -> String {
        let cooked = cook_order(); // relative: a sibling in this module
        // `super` = the parent module (here the crate root), like `..` in a
        // file system path.
        format!("{cooked}, then {}", super::deliver_order())
    }

    fn cook_order() -> &'static str {
        "re-cooked"
    }

    pub mod kitchen {
        pub fn ask_front() -> &'static str {
            // Go up two levels, then down: kitchen -> back_of_house -> crate.
            super::super::front_of_house::hosting::add_to_waitlist()
        }
    }
}

fn main() {
    // Absolute path: from the crate root. Keeps working if THIS code moves.
    println!("{}", crate::front_of_house::hosting::add_to_waitlist());

    // Relative path: from the current module. Keeps working if this code and
    // the target move together.
    println!("{}", front_of_house::hosting::add_to_waitlist());

    println!("{}", back_of_house::fix_incorrect_order());
    println!("{}", back_of_house::kitchen::ask_front());

    // The Book's advice: prefer absolute paths, because code and the items it
    // calls are more often moved independently.
}
