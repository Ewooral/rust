//! Beyond the Book: PhantomData, variance, and the orphan rule.
//! Run: cargo run -p generics_traits_lifetimes --example variance_and_orphan_rule

use std::fmt;
use std::marker::PhantomData;

// ---- PhantomData: a type parameter with no field that uses it ----
// Unit markers: zero-size types that exist only as labels.
#[derive(Debug, Clone, Copy)]
struct Metres;
#[derive(Debug, Clone, Copy)]
struct Feet;

#[derive(Debug, Clone, Copy)]
struct Length<Unit> {
    value: f64,
    unit: PhantomData<Unit>, // zero bytes; tells the compiler "I am tied to Unit"
}

impl<Unit> Length<Unit> {
    fn new(value: f64) -> Self {
        Length {
            value,
            unit: PhantomData,
        }
    }
    fn add(self, other: Length<Unit>) -> Length<Unit> {
        Length::new(self.value + other.value)
    }
}

impl Length<Feet> {
    fn to_metres(self) -> Length<Metres> {
        Length::new(self.value * 0.3048)
    }
}

// ---- Variance: when may one lifetime stand in for another? ----
// &'a T is COVARIANT in 'a: a longer-lived reference can be used where a
// shorter one is expected. That is why a &'static str fits any &'a str.
fn pick<'a>(x: &'a str, _y: &'a str) -> &'a str {
    x
}

// &'a mut T is INVARIANT in T: no substitution at all. If it were not, you
// could store a short-lived reference into a slot that promises a long one:
//
//     fn overwrite<'a>(slot: &mut &'static str, short: &'a str) {
//         *slot = short; // error: lifetime may not live long enough
//     }
//
// The compiler works variance out for you from a type's fields; PhantomData
// is how you tell it the right variance when there is no real field.

// ---- The orphan rule (coherence) ----
// You may implement a trait for a type only if the TRAIT or the TYPE is
// defined in your crate. Both Display and Vec come from std, so:
//
//     impl fmt::Display for Vec<String> { ... }
//     // E0117: only traits defined in the current crate can be implemented
//     //        for types defined outside of the crate
//
// The fix is the newtype pattern: wrap the foreign type in a local one.
struct List(Vec<String>);

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}
// Why the rule exists: if two crates could both implement Display for
// Vec<String>, the compiler could not know which one to use.

fn main() {
    let a: Length<Metres> = Length::new(2.0);
    let b: Length<Metres> = Length::new(3.0);
    let c: Length<Feet> = Length::new(10.0);
    println!("a + b = {} m", a.add(b).value);
    println!("10 ft = {:.3} m", c.to_metres().value);
    // a.add(c); // E0308: expected `Length<Metres>`, found `Length<Feet>`
    println!(
        "Length<Metres> is {} bytes, same as f64",
        size_of::<Length<Metres>>()
    );

    let forever: &'static str = "static text";
    let local = String::from("local text");
    println!(
        "covariance lets 'static shrink to fit: {}",
        pick(forever, &local)
    );

    let list = List(vec!["ama".into(), "kofi".into()]);
    println!("newtype Display: {list}");
}
