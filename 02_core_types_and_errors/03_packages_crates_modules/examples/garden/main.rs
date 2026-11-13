//! Separating modules into files: the Book's garden example.
//! Run: cargo run -p packages_crates_modules --example garden
//!
//! A directory in examples/ with a main.rs becomes one example. Its layout:
//!
//!   examples/garden/main.rs               crate root (this file)
//!   examples/garden/garden.rs             mod garden
//!   examples/garden/garden/vegetables.rs  mod garden::vegetables

// `mod garden;` with a semicolon: "load the module body from garden.rs".
// You write `mod` ONCE per module in the tree. Everywhere else, use a path.
pub mod garden;

use crate::garden::vegetables::Asparagus;

fn main() {
    let plant = Asparagus::new(12);
    println!("I'm growing {plant:?}!");
    println!("ready to pick? {}", plant.is_ready());
    println!("{}", garden::describe());
}
