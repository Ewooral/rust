//! Book 14.2: exporting a convenient public API with `pub use`.
//!
//! Run: cargo run -p cargo_and_crates_io --example pub_use_reexports
//!
//! The way YOU organise a library (modules inside modules) is not always the
//! way your USERS want to find things. `pub use` re-exports an item at a
//! shorter path, without moving it.

// The Book's `art` crate, written as a module so it fits in one file.
pub mod art {
    //! A library for modeling artistic concepts.

    // Without these three lines, users must write `art::kinds::PrimaryColor`
    // and `art::utils::mix`. With them, `art::PrimaryColor` and `art::mix`.
    pub use self::kinds::PrimaryColor;
    pub use self::kinds::SecondaryColor;
    pub use self::utils::mix;

    /// The kinds of colour.
    pub mod kinds {
        /// The primary colors according to the RYB color model.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum PrimaryColor {
            /// Red.
            Red,
            /// Yellow.
            Yellow,
            /// Blue.
            Blue,
        }

        /// The secondary colors according to the RYB color model.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum SecondaryColor {
            /// Red and yellow.
            Orange,
            /// Yellow and blue.
            Green,
            /// Red and blue.
            Purple,
        }
    }

    /// Things you can do with colours.
    pub mod utils {
        use super::kinds::{PrimaryColor, SecondaryColor};

        /// Combines two primary colors in equal amounts to create a
        /// secondary color. Mixing a colour with itself gives `None`.
        pub fn mix(c1: PrimaryColor, c2: PrimaryColor) -> Option<SecondaryColor> {
            use PrimaryColor::{Blue, Red, Yellow};
            match (c1, c2) {
                (Red, Yellow) | (Yellow, Red) => Some(SecondaryColor::Orange),
                (Yellow, Blue) | (Blue, Yellow) => Some(SecondaryColor::Green),
                (Red, Blue) | (Blue, Red) => Some(SecondaryColor::Purple),
                _ => None,
            }
        }
    }
}

// The user's side: short paths, thanks to the re-exports.
use art::PrimaryColor;
use art::mix;

fn main() {
    let red = PrimaryColor::Red;
    let yellow = PrimaryColor::Yellow;
    println!("red + yellow = {:?}", mix(red, yellow));
    println!("red + red    = {:?}", mix(red, red));

    // The long path still works: re-exporting adds a name, it moves nothing.
    let blue = art::kinds::PrimaryColor::Blue;
    println!("red + blue   = {:?}", art::utils::mix(red, blue));

    // Both paths name the SAME type. Its "real" path is where it is defined:
    println!(
        "\nart::PrimaryColor is really {}",
        std::any::type_name::<art::PrimaryColor>()
    );
    // In `cargo doc`, the crate's front page lists the re-exports, so users
    // find PrimaryColor and mix without digging through the modules.
}
