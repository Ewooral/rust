//! Solution to exercise 2: make it compile (unused_must_use), and make it WORK.
//!
//! The programmer expected `map` to double every price. But iterator
//! adapters are lazy: nothing happens until something consumes the
//! iterator. Normally that is a warning; this file denies it, so it's an
//! error. Collect the doubled prices into `doubled` and print that.
//!
//! Run: cargo run -p iterators_and_closures --features solutions --bin sol02_fix_lazy_iterator

#![deny(unused_must_use)]

fn main() {
    let prices = vec![120, 95, 300];
    let doubled: Vec<i32> = prices.iter().map(|p| p * 2).collect();
    println!("{prices:?} -> {doubled:?}");
}
