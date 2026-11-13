//! Book 10.3: validating references with lifetimes.
//! Run: cargo run -p generics_traits_lifetimes --example lifetimes

use std::fmt::Display;

// ---- Lifetime annotations in functions ----
// `'a` is a generic LIFETIME parameter. This signature says: the returned
// reference is valid for as long as BOTH x and y are valid, i.e. for the
// shorter of the two. It changes nothing at run time; it describes a
// relationship so the borrow checker can check callers.
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
// Without 'a: E0106 missing lifetime specifier
// ("this function's return type contains a borrowed value, but the signature
//  does not say whether it is borrowed from `x` or `y`").

// The result only ever comes from x, so y needs no relationship with it.
fn first<'a>(x: &'a str, _y: &str) -> &'a str {
    x
}

// ---- Lifetime annotations in structs ----
// An ImportantExcerpt cannot outlive the text its `part` points into.
#[derive(Debug)]
struct ImportantExcerpt<'a> {
    part: &'a str,
}

impl<'a> ImportantExcerpt<'a> {
    // Elision rule 3: with &self, the output gets self's lifetime.
    fn announce_and_return_part(&self, announcement: &str) -> &str {
        println!("Attention please: {announcement}");
        self.part
    }
}

// ---- Lifetime elision: you often write none ----
// Rule 1: each reference parameter gets its own lifetime.
// Rule 2: if there is exactly ONE input lifetime, outputs get it.
// Rule 3: if one input is &self or &mut self, outputs get self's lifetime.
// So this needs no annotations (rule 2):
fn first_word(s: &str) -> &str {
    s.split(' ').next().unwrap_or("")
}

// ---- Generics, trait bounds and lifetimes together ----
fn longest_with_an_announcement<'a, T>(x: &'a str, y: &'a str, ann: T) -> &'a str
where
    T: Display,
{
    println!("Announcement! {ann}");
    if x.len() > y.len() { x } else { y }
}

fn main() {
    // The borrow checker compares scopes. This would not compile:
    //     let r;
    //     {
    //         let x = 5;
    //         r = &x;      // E0597: `x` does not live long enough
    //     }
    //     println!("r: {r}");
    let x = 5;
    let r = &x; // fine: x outlives r
    println!("r: {r}");

    let string1 = String::from("long string is long");
    let result;
    {
        let string2 = String::from("xyz");
        let inner = longest(string1.as_str(), string2.as_str());
        println!("The longest string is {inner}"); // fine: used while string2 lives
        result = first(string1.as_str(), string2.as_str()); // only tied to string1
    }
    println!("first is still valid: {result}");

    let novel = String::from("Call me Ishmael. Some years ago...");
    let first_sentence = novel.split('.').next().unwrap_or("");
    let excerpt = ImportantExcerpt {
        part: first_sentence,
    };
    println!("{excerpt:?}");
    println!("{}", excerpt.announce_and_return_part("today's quote"));
    println!("first word: {}", first_word(&novel));

    // 'static: lives for the whole program. String literals are &'static str,
    // stored in the binary itself.
    let s: &'static str = "I have a static lifetime.";
    println!("{s}");
    // If the compiler suggests adding 'static, first ask: is the data REALLY
    // meant to live forever, or am I returning a reference to a local?

    println!(
        "{}",
        longest_with_an_announcement("short", "much longer", "here it comes")
    );
}
