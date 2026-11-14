//! An attribute macro: `#[memoize]` takes a function and gives back a new
//! one that remembers its results.
//!
//! Run: cargo run -p macros_in_depth --example attribute_memoize
//!
//! The macro is in `macros/src/memoize.rs`.

use std::time::Instant;

use macros_in_depth::memoize;

/// Without `#[memoize]` this takes about 2^n calls.
#[memoize]
fn fibonacci(n: u64) -> u128 {
    if n < 2 {
        return n.into();
    }
    fibonacci(n - 1) + fibonacci(n - 2)
}

/// Arguments become the cache key: `(String, usize)` here.
/// `capacity = 2` empties the cache whenever it holds 2 entries.
#[memoize(capacity = 2)]
fn repeat(word: String, times: usize) -> String {
    println!("    (really computing repeat({word:?}, {times}))");
    word.repeat(times)
}

// The new function, roughly (`cargo expand` shows the real thing):
//
//     fn fibonacci(n: u64) -> u128 {
//         thread_local! { static __MEMOIZE_CACHE: RefCell<HashMap<(u64,), u128>> = … }
//         let key = (n.clone(),);
//         if let Some(hit) = __MEMOIZE_CACHE.with(|c| c.borrow().get(&key).cloned()) {
//             return hit;
//         }
//         let result: u128 = (move || -> u128 { /* the original body */ })();
//         __MEMOIZE_CACHE.with(|c| c.borrow_mut().insert(key, result.clone()));
//         result
//     }
//
// What it refuses, each error pointing at the part that's wrong:
//
//     #[memoize] async fn load(id: u32) -> String { … }
//     error: #[memoize] can't cache an `async fn`: its result is a future
//
//     #[memoize] fn length(text: &str) -> usize { … }
//     error: #[memoize] keeps arguments in its cache, so they must be owned:
//            use `String` instead of `&str`, `Vec<T>` instead of `&[T]`
//
//     #[memoize] fn log(line: String) { … }
//     error: #[memoize] needs a function that returns a value: there is nothing to remember
//
//     #[memoize(size = 10)]
//     error: unknown memoize option: the only one is `capacity = <number>`
//
//     #[memoize] fn area(shape: Shape) -> f64     (Shape without Hash)
//     error[E0277]: the trait bound `Shape: Hash` is not satisfied
//     It points at `Shape` in the signature, thanks to `quote_spanned!`.
//     Two follow-on E0599 errors ("the method `get` exists … but its trait
//     bounds were not satisfied") point at `#[memoize]`; they go away with
//     the first one.

fn main() {
    let start = Instant::now();
    println!("fibonacci(150) = {}", fibonacci(150));
    println!("  took {:?}: each n was computed once", start.elapsed());

    println!("repeat calls:");
    for (word, times) in [("ab", 2), ("ab", 2), ("cd", 3), ("ef", 1), ("ab", 2)] {
        println!(
            "  repeat({word:?}, {times}) = {}",
            repeat(word.into(), times)
        );
    }
}
