//! `pub use`: re-exporting, seen from OUTSIDE the library.
//! Run: cargo run -p packages_crates_modules --example re_exports
//!
//! Examples are separate crates that depend on this package's library, so
//! this file sees `restaurant` exactly as a stranger would.

// Inside the library, `hosting` really lives at
// `restaurant::front_of_house::hosting`, but `front_of_house` is private.
// src/lib.rs says `pub use crate::front_of_house::hosting;`, so the public
// path is shorter:
use restaurant::hosting;

// use restaurant::front_of_house::hosting; // E0603: module `front_of_house` is private

fn main() {
    let mut list = hosting::Waitlist::new();
    hosting::add_to_waitlist(&mut list, "Ama");
    println!("waiting: {}", list.len());
    println!("{:?}", hosting::seat_at_table(&mut list));

    // `pub use` lets a library present a tidy public API (what users need)
    // that is different from its internal structure (how it is built).
}
