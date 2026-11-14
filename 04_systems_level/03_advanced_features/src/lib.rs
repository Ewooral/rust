//! advanced_features: a small, typed units-and-vectors toolkit that puts the
//! tools of Book chapter 20 to real work.
//!
//! | Module / item | Chapter 20 tools it uses |
//! |---|---|
//! | [`units`] | newtypes made by a declarative macro, operator overloading, a default type parameter (`Meters + Millimeters`) |
//! | [`geometry`] | `Vec2` operators (including `f64 * Vec2`), an iterator with an associated type, a supertrait |
//! | [`transform`] | a type alias for a function pointer, a lookup table of `fn`s, functions that return closures |
//! | [`raw`] | a tiny, audited `unsafe` block behind a safe API (`split_at_mut`) |
//! | [`trail!`] | a `vec!`-style declarative macro |
//! | [`HelloMacro`] | a trait plus a custom derive from our own proc-macro crate |
//!
//! Run the tour with `cargo run -p advanced_features`.

use std::fmt;

pub mod geometry;
pub mod raw;
pub mod transform;
pub mod units;

pub use geometry::{Segments, Trail, Vec2};
pub use units::{Meters, MetersPerSecond, Millimeters, Seconds};

/// The Book's `HelloMacro` trait (20.5). `#[derive(HelloMacro)]` implements it.
///
/// The trait and the derive macro share a name, like `serde::Serialize`: a
/// trait lives in the *type* namespace and a derive in the *macro* namespace,
/// so `use advanced_features::HelloMacro;` imports both at once.
pub trait HelloMacro {
    fn hello_macro();
}

// Re-export the procedural macros so users need only this crate.
pub use advanced_features_macros::{HelloMacro, shout, trace};

/// The Book's supertrait example (20.2): anything that can be displayed can
/// be printed inside a frame of stars.
///
/// `OutlinePrint: fmt::Display` means "to implement `OutlinePrint`, you must
/// already implement `Display`", so the default methods may use `{self}`.
pub trait OutlinePrint: fmt::Display {
    /// The framed text, so it can be tested as well as printed.
    fn outline(&self) -> String {
        let text = self.to_string();
        let width = text.chars().count();
        let stars = "*".repeat(width + 4);
        let blank = format!("*{}*", " ".repeat(width + 2));
        format!("{stars}\n{blank}\n* {text} *\n{blank}\n{stars}")
    }

    fn outline_print(&self) {
        println!("{}", self.outline());
    }
}

/// Builds a [`Trail`] from `(x, y)` pairs, like `vec!` builds a `Vec`.
///
/// ```
/// use advanced_features::trail;
///
/// let walk = trail![(0, 0), (3, 4), (3.5, 4)];
/// assert_eq!(walk.points().len(), 3);
/// assert_eq!(walk.length().0, 5.5);
/// ```
///
/// - Rules are tried top to bottom. The first handles `trail![]`.
/// - `$( … ),*` repeats the pattern for each comma-separated pair, and the
///   matching `$( … )*` in the body repeats the code once per pair.
/// - `$(,)?` allows an optional trailing comma.
/// - `$crate::` always names THIS crate, even when the macro is used in
///   another crate that has renamed or not imported it.
/// - `f64::from` accepts integers and floats, so `(3, 4)` and `(3.5, 4)` both work.
#[macro_export]
macro_rules! trail {
    () => {
        $crate::Trail::new()
    };
    ( $( ($x:expr, $y:expr) ),+ $(,)? ) => {{
        let mut trail = $crate::Trail::new();
        $( trail.push($crate::Vec2::new(f64::from($x), f64::from($y))); )*
        trail
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trail_macro_builds_in_order_and_accepts_trailing_comma() {
        let walk = trail![(0, 0), (1.5, 2), (3, 4),];
        assert_eq!(
            walk.points(),
            [
                Vec2::new(0.0, 0.0),
                Vec2::new(1.5, 2.0),
                Vec2::new(3.0, 4.0)
            ]
        );
        let empty = trail![];
        assert!(empty.is_empty());
    }

    #[test]
    fn outline_frames_any_display_type() {
        struct Name;
        impl fmt::Display for Name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "Ama")
            }
        }
        impl OutlinePrint for Name {}
        assert_eq!(
            Name.outline(),
            "*******\n*     *\n* Ama *\n*     *\n*******"
        );
    }

    #[test]
    fn derive_hello_macro_implements_the_trait() {
        #[derive(HelloMacro)]
        struct Pancakes;
        Pancakes::hello_macro(); // compiles only because the derive wrote the impl
    }
}
