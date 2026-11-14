//! Beyond the Book: `LazyLock` and `OnceLock` for lazy global values.
//!
//! Run: cargo run -p smart_pointers --example lazy_globals
//!
//! A `static` must be built at compile time, so it can't hold a HashMap or
//! anything read from a file. These two types build the value the first
//! time it is used, exactly once, even if several threads ask at the same
//! moment.

use std::collections::HashMap;
use std::sync::{LazyLock, OnceLock};
use std::thread;

// LazyLock: the recipe is written next to the static. It runs on first use.
static CAPITALS: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    println!("  (building CAPITALS: this happens once)");
    HashMap::from([("Ghana", "Accra"), ("Kenya", "Nairobi"), ("Peru", "Lima")])
});

// OnceLock: empty until the program sets it, at run time, once.
static CONFIG: OnceLock<String> = OnceLock::new();

fn config() -> &'static str {
    // Set it if nobody has yet, then return it.
    CONFIG.get_or_init(|| String::from("default config"))
}

fn main() {
    println!("main starts; CAPITALS hasn't been built yet");
    println!("capital of Ghana: {}", CAPITALS["Ghana"]);
    println!("capital of Peru:  {}", CAPITALS["Peru"]); // no second build

    // OnceLock::set succeeds the first time only.
    println!(
        "\nset CONFIG: {:?}",
        CONFIG.set(String::from("from the command line"))
    );
    println!("set again:  {:?}", CONFIG.set(String::from("too late")));
    println!("config():   {}", config());

    // Both are thread-safe: four threads read the same map, built once.
    let handles: Vec<_> = ["Kenya", "Ghana", "Peru", "Kenya"]
        .into_iter()
        .map(|country| thread::spawn(move || CAPITALS[country]))
        .collect();
    let answers: Vec<&str> = handles
        .into_iter()
        .map(|h| h.join().expect("the thread finished"))
        .collect();
    println!("\nfrom four threads: {answers:?}");
}

// Single-thread versions: std::cell::LazyCell and std::cell::OnceCell.
// Before Rust 1.80, people used the `once_cell` or `lazy_static` crates.
