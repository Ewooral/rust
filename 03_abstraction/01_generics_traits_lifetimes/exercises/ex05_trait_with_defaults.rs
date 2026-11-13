//! Exercise 5: define behaviour with a trait, use it statically and dynamically.
//!
//! 1. Implement `Animal` for `Dog` and `Cow`. Only `name` and `sound` are
//!    required. Keep the default `legs` and `speak` for Dog. For Cow, keep
//!    the default `legs` but override `speak` to return
//!    "The cow says Moo, slowly".
//! 2. `speak_all` is generic (static dispatch). `chorus` takes trait objects
//!    (dynamic dispatch), so one Vec can hold dogs and cows together.
//!
//! Test: cargo test -p generics_traits_lifetimes --features exercises --bin ex05_trait_with_defaults

#![allow(unused_variables, dead_code)]

trait Animal {
    fn name(&self) -> String;
    fn sound(&self) -> String;

    fn legs(&self) -> u32 {
        4
    }

    fn speak(&self) -> String {
        format!("The {} says {}", self.name(), self.sound())
    }
}

struct Dog;
struct Cow;

// TODO: impl Animal for Dog  (name "dog", sound "Woof")
// TODO: impl Animal for Cow  (name "cow", sound "Moo", custom speak)

/// Static dispatch: every item is the same type T.
fn speak_all<T: Animal>(animals: &[T]) -> Vec<String> {
    todo!()
}

/// Dynamic dispatch: a mix of types behind `dyn Animal`.
fn chorus(animals: &[Box<dyn Animal>]) -> String {
    // join each animal's speak() with " / "
    todo!()
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_overrides() {
        assert_eq!(Dog.speak(), "The dog says Woof");
        assert_eq!(Cow.speak(), "The cow says Moo, slowly");
        assert_eq!(Dog.legs() + Cow.legs(), 8);
    }

    #[test]
    fn static_dispatch() {
        assert_eq!(
            speak_all(&[Dog, Dog]),
            ["The dog says Woof", "The dog says Woof"]
        );
    }

    #[test]
    fn dynamic_dispatch() {
        let farm: Vec<Box<dyn Animal>> = vec![Box::new(Dog), Box::new(Cow)];
        assert_eq!(
            chorus(&farm),
            "The dog says Woof / The cow says Moo, slowly"
        );
    }
}
