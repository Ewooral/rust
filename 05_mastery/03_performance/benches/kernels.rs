//! Micro-benchmarks: data layout, iterators vs loops, bounds checks.
//!
//! Run: cargo bench -p performance --bench kernels
//! One group: cargo bench -p performance --bench kernels -- layout

use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use performance::kernels::{self, Particles};

fn layout(c: &mut Criterion) {
    // A million particles: 32 MB as structs, but only 4 MB of masses.
    let aos = kernels::make_particles(1_000_000);
    let soa = Particles::from_aos(&aos);
    let mut group = c.benchmark_group("layout");
    group.bench_function("array_of_structs", |b| {
        b.iter(|| kernels::count_heavy_aos(black_box(&aos), 4.0))
    });
    group.bench_function("struct_of_arrays", |b| {
        b.iter(|| kernels::count_heavy_soa(black_box(&soa), 4.0))
    });
    group.finish();
}

fn loops(c: &mut Criterion) {
    let values: Vec<u64> = (0..100_000).collect();
    let mut group = c.benchmark_group("loops");
    group.bench_function("index_loop", |b| {
        b.iter(|| kernels::sum_even_squares_loop(black_box(&values)))
    });
    group.bench_function("iterator", |b| {
        b.iter(|| kernels::sum_even_squares_iter(black_box(&values)))
    });
    group.finish();
}

fn bounds(c: &mut Criterion) {
    let a: Vec<f32> = (0..100_000).map(|i| i as f32).collect();
    let b_values = vec![0.5; 100_000];
    let mut out = vec![0.0; 100_000];
    let mut group = c.benchmark_group("bounds");
    group.bench_function("indexed", |b| {
        b.iter(|| kernels::add_indexed(black_box(&a), black_box(&b_values), black_box(&mut out)))
    });
    group.bench_function("resliced", |b| {
        b.iter(|| kernels::add_resliced(black_box(&a), black_box(&b_values), black_box(&mut out)))
    });
    group.bench_function("zip", |b| {
        b.iter(|| kernels::add_zip(black_box(&a), black_box(&b_values), black_box(&mut out)))
    });
    group.finish();
}

fn quick() -> Criterion {
    Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_millis(200))
        .measurement_time(Duration::from_millis(500))
}

criterion_group! {
    name = benches;
    config = quick();
    targets = layout, loops, bounds
}
criterion_main!(benches);
