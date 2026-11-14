//! Cache-friendly data layout: padding, and array of structs vs struct of arrays.
//!
//! Run: cargo run -p performance --example data_layout --release

use std::mem::size_of;

use performance::kernels::{self, Particle, Particles};
use performance::timing::{build_kind, median_time};

// Rust may reorder fields to waste less space on padding...
#[allow(dead_code)]
struct Reordered {
    a: u8,
    b: u64,
    c: u8,
}

// ...unless you ask for C's layout, which keeps your order.
#[allow(dead_code)]
#[repr(C)]
struct InDeclaredOrder {
    a: u8,
    b: u64,
    c: u8,
}

fn main() {
    println!("{}\n", build_kind());

    println!("Padding: the same three fields (u8, u64, u8)");
    println!("  default layout: {:>2} bytes", size_of::<Reordered>());
    println!(
        "  #[repr(C)]:     {:>2} bytes\n",
        size_of::<InDeclaredOrder>()
    );

    let n = 1_000_000;
    let aos = kernels::make_particles(n);
    let soa = Particles::from_aos(&aos);
    println!("{n} particles, {} bytes each", size_of::<Particle>());
    println!(
        "  Vec<Particle> (array of structs): {:>2} MB pass through the cache",
        n * size_of::<Particle>() / 1_000_000
    );
    println!(
        "  Particles.mass (struct of arrays): {:>1} MB pass through the cache\n",
        n * size_of::<f32>() / 1_000_000
    );

    assert_eq!(
        kernels::count_heavy_aos(&aos, 4.0),
        kernels::count_heavy_soa(&soa, 4.0)
    );
    let aos_time = median_time(5, || kernels::count_heavy_aos(&aos, 4.0));
    let soa_time = median_time(5, || kernels::count_heavy_soa(&soa, 4.0));
    println!("count the particles heavier than 4.0:");
    println!("  array of structs {aos_time:>10.2?}");
    println!("  struct of arrays {soa_time:>10.2?}");
}
