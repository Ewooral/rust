//! Beyond the Book: lifetime bounds, the anonymous lifetime '_, 'static as a
//! bound, and higher-ranked trait bounds (for<'a>).
//! Run: cargo run -p generics_traits_lifetimes --example advanced_lifetimes

use std::fmt::Debug;

// ---- 'a: 'b  ("'a outlives 'b") ----
// We hold a reference that lives for 'b, taken from data that lives for 'a.
// Valid only if 'a lasts at least as long as 'b.
struct Context<'a> {
    text: &'a str,
}

struct Parser<'c, 's> {
    context: &'c Context<'s>,
}

impl<'c, 's: 'c> Parser<'c, 's> {
    // We can return &'s str (the longer lifetime), not just &'c str, because
    // the text lives in the Context's source, not in the Parser.
    fn first_word(&self) -> &'s str {
        self.context.text.split(' ').next().unwrap_or("")
    }
}

fn parse_first(context: &Context<'_>) -> String {
    Parser { context }.first_word().to_string()
}

// ---- T: 'a  ("T contains no references shorter than 'a") ----
struct Holder<'a, T: 'a> {
    item: &'a T,
}

// ---- The anonymous lifetime '_ ----
// "There is a lifetime here; let elision choose it." Used to make a hidden
// borrow visible without naming it:
struct Words<'a> {
    text: &'a str,
}
fn words(text: &str) -> Words<'_> {
    // clearer than `-> Words`, which hides that it borrows `text`
    Words { text }
}

// ---- 'static as a BOUND ----
// `T: 'static` does NOT mean "T lives forever". It means T holds no
// borrowed data shorter than 'static: an OWNED String qualifies.
// Threads require this (lesson 04/01): a thread may outlive the caller.
fn keep_forever<T: Debug + Send + 'static>(value: T) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || println!("thread got {value:?}"))
}

// ---- Higher-ranked trait bounds: for<'a> ----
// "f must work for EVERY lifetime 'a the caller might pick", not for one
// fixed lifetime chosen up front. Needed here because each call passes a
// reference to a different local String.
fn apply_to_each<F>(lines: &[&str], f: F) -> Vec<String>
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    lines
        .iter()
        .map(|line| {
            let owned = line.trim().to_string(); // a fresh local each time
            f(&owned).to_string()
        })
        .collect()
}

fn first_token(s: &str) -> &str {
    s.split(' ').next().unwrap_or("")
}

fn main() {
    let ctx = Context {
        text: "hello lifetimes world",
    };
    println!("parsed: {}", parse_first(&ctx));

    let n = 7;
    let h = Holder { item: &n };
    println!("holder: {}", h.item);

    let w = words("borrowed text");
    println!("words view: {}", w.text);

    let owned = String::from("an owned String is 'static");
    keep_forever(owned).join().expect("thread finished");
    // let local = String::from("x");
    // keep_forever(&local); // E0597: `local` does not live long enough
    //                       // (a &String borrowed from here is not 'static)

    // Elided fn signatures like `fn(&str) -> &str` are already higher-ranked.
    println!(
        "{:?}",
        apply_to_each(&[" alpha beta ", "gamma"], first_token)
    );
}
