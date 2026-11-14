//! Iterators are a zero-cost abstraction: as fast as the loop you'd write by hand.
//!
//! Run: cargo run -p performance --example iterators_vs_loops --release

use performance::kernels;
use performance::timing::{build_kind, median_time};

/// A third way: a `for` loop over the items (no index).
fn sum_even_squares_for(values: &[u64]) -> u64 {
    let mut total = 0;
    for &v in values {
        if v.is_multiple_of(2) {
            total += v * v;
        }
    }
    total
}

fn main() {
    println!("{}\n", build_kind());
    let values: Vec<u64> = (0..1_000_000).collect();

    let while_loop = kernels::sum_even_squares_loop(&values);
    let for_loop = sum_even_squares_for(&values);
    let iterator = kernels::sum_even_squares_iter(&values);
    assert!(while_loop == for_loop && for_loop == iterator);
    println!("sum of the squares of the even numbers below 1 000 000 = {iterator}\n");

    let t_while = median_time(9, || kernels::sum_even_squares_loop(&values));
    let t_for = median_time(9, || sum_even_squares_for(&values));
    let t_iter = median_time(9, || kernels::sum_even_squares_iter(&values));
    println!("while + index    {t_while:>10.2?}   values[i], i += 1");
    println!("for over items   {t_for:>10.2?}   for &v in values");
    println!("iterator chain   {t_iter:>10.2?}   filter(..).map(..).sum()");
}
