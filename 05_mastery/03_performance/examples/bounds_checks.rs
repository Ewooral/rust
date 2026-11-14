//! Bounds checks: what `v[i]` costs, and three ways to avoid paying it.
//!
//! Run: cargo run -p performance --example bounds_checks --release

use performance::kernels;
use performance::timing::{build_kind, median_time};

fn main() {
    println!("{}\n", build_kind());

    // Every `v[i]` checks `i < v.len()` and panics if not: that check is
    // what keeps safe Rust free of buffer overflows. `get` does the same
    // check but gives you an Option instead of a panic.
    let v = [10, 20, 30];
    println!("v.get(1) = {:?}, v.get(7) = {:?}", v.get(1), v.get(7));
    // let x = v[7];
    // An array with a constant index is caught while compiling (this error
    // has no code; it is the deny-by-default `unconditional_panic` lint):
    //   error: this operation will panic at runtime
    // With a Vec, or an index only known at run time, it compiles and then
    // panics: index out of bounds: the len is 3 but the index is 7

    // 4 096 floats per slice: small enough to stay in the CPU's fastest
    // caches, so we time the loop itself and not the memory bus.
    let n = 4_096;
    let a: Vec<f32> = (0..n).map(|i| i as f32).collect();
    let b = vec![0.5f32; n];
    let mut out = vec![0.0f32; n];
    let mut time = |add: fn(&[f32], &[f32], &mut [f32])| {
        median_time(9, || {
            for _ in 0..1_000 {
                add(&a, &b, &mut out);
            }
        })
    };

    let indexed = time(kernels::add_indexed);
    let resliced = time(kernels::add_resliced);
    let zipped = time(kernels::add_zip);
    println!("\nout[i] = a[i] + b[i] for {n} floats, 1 000 times:");
    println!("  index all three slices          {indexed:>10.2?}");
    println!("  re-slice to len first, then [i] {resliced:>10.2?}");
    println!("  zip, no indexes at all          {zipped:>10.2?}");
}
