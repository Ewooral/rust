//! `#[inline]`: letting the optimiser see inside a call.
//!
//! Run: cargo run -p performance --example inlining --release

use std::hint::black_box;

use performance::timing::{build_kind, median_time};

// The compiler decides by itself which calls to inline. These attributes
// override its choice, so we can see what inlining is worth.
#[inline(never)]
fn scale_never(x: f32) -> f32 {
    x * 1.5 + 2.0
}

#[inline(always)]
fn scale_always(x: f32) -> f32 {
    x * 1.5 + 2.0
}

fn main() {
    println!("{}\n", build_kind());
    let input: Vec<f32> = (0..1_000_000).map(|i| i as f32).collect();
    let mut out = vec![0.0f32; input.len()];

    let never = median_time(9, || {
        for (o, &x) in out.iter_mut().zip(&input) {
            *o = scale_never(x); // a real call for every element
        }
        // Without this the optimiser sees that nobody reads `out` and
        // deletes the whole loop (try it: both times drop to ~0).
        black_box(&out);
    });
    let always = median_time(9, || {
        for (o, &x) in out.iter_mut().zip(&input) {
            *o = scale_always(x); // the body is pasted in, so SIMD is possible
        }
        black_box(&out);
    });
    println!("1 000 000 calls of a tiny function:");
    println!("  #[inline(never)]  {never:>10.2?}");
    println!("  #[inline(always)] {always:>10.2?}");
}
