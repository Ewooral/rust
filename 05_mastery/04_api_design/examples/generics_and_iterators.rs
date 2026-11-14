//! Flexible arguments (`impl AsRef<Path>`, `impl Into<String>`,
//! `impl IntoIterator`) and returning iterators.
//!
//! Run: cargo run -p api_design --example generics_and_iterators

use std::path::{Path, PathBuf};

/// `impl AsRef<Path>`: callers pass `&str`, `String`, `&Path` or `PathBuf`,
/// as with `std::fs::read_to_string`.
fn extension_of(path: impl AsRef<Path>) -> String {
    // Each type the caller uses gets its OWN copy of a generic function. Keep
    // that copy tiny: convert once, then call a plain (non-generic) inner
    // function that does the work and is compiled only once.
    fn inner(path: &Path) -> String {
        path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("(none)")
            .to_string()
    }
    inner(path.as_ref())
}

/// `impl Into<String>`: the function needs to OWN a String. A caller with a
/// `&str` lets us copy it; a caller with a `String` they don't need any
/// more hands it over, and nothing is copied.
struct Label {
    text: String,
}

impl Label {
    fn new(text: impl Into<String>) -> Self {
        Label { text: text.into() }
    }
}

/// `impl IntoIterator<Item = …>`: a Vec, an array, a slice iterator, a
/// `map(…)`: anything you can loop over.
fn total_length<I>(words: I) -> usize
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    words.into_iter().map(|w| w.as_ref().len()).sum()
}

struct Inventory {
    items: Vec<(String, u32)>,
}

impl Inventory {
    /// Returning `impl Iterator`: the caller can loop, filter, collect…,
    /// and we keep the freedom to change how it's computed. The `+ '_` says
    /// it borrows `self`.
    fn in_stock(&self) -> impl Iterator<Item = &str> + '_ {
        self.items
            .iter()
            .filter(|(_, count)| *count > 0)
            .map(|(name, _)| name.as_str())
    }

    /// Different iterators in different branches have different types:
    ///     if only_low { a.filter(…) } else { a }
    ///     error[E0308]: `if` and `else` have incompatible types
    /// A `Box<dyn Iterator>` is one type for both (one allocation).
    fn names(&self, only_low: bool) -> Box<dyn Iterator<Item = &str> + '_> {
        let all = self.items.iter();
        if only_low {
            Box::new(all.filter(|(_, n)| *n < 5).map(|(name, _)| name.as_str()))
        } else {
            Box::new(all.map(|(name, _)| name.as_str()))
        }
    }
}

/// `for (name, count) in &inventory` works once `&Inventory: IntoIterator`.
impl<'a> IntoIterator for &'a Inventory {
    type Item = &'a (String, u32);
    type IntoIter = std::slice::Iter<'a, (String, u32)>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

fn main() {
    let owned = PathBuf::from("notes/today.md");
    println!(
        "{} {} {} {}",
        extension_of("photo.jpeg"),
        extension_of(String::from("data.csv")),
        extension_of(Path::new("Cargo.toml")),
        extension_of(&owned)
    );

    let borrowed = Label::new("from a &str"); // copied
    let name = String::from("from a String");
    let moved = Label::new(name); // moved, no copy (`name` is gone now)
    println!("{} / {}", borrowed.text, moved.text);

    println!(
        "total_length: {} {} {}",
        total_length(["a", "bb"]),
        total_length(vec![String::from("ccc")]),
        total_length(["x", "yy", "zzz"].iter().filter(|w| w.len() > 1))
    );

    let inventory = Inventory {
        items: vec![
            ("apples".into(), 12),
            ("pears".into(), 0),
            ("plums".into(), 3),
        ],
    };
    let stocked: Vec<&str> = inventory.in_stock().collect();
    println!("in stock: {stocked:?}");
    println!("low:      {:?}", inventory.names(true).collect::<Vec<_>>());
    for (name, count) in &inventory {
        println!("  {name}: {count}");
    }

    // The library returns a NAMED iterator type instead, which can promise
    // more: `HeaderIter` is ExactSizeIterator and DoubleEndedIterator.
    let request = api_design::Request::builder()
        .url("http://example.org/")
        .header("Accept", "*/*")
        .build()
        .unwrap();
    let headers = request.headers().iter();
    println!(
        "{} headers, last first: {:?}",
        headers.len(),
        headers.rev().collect::<Vec<_>>()
    );
}
