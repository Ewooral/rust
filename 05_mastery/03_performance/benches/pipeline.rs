//! Benchmarks for the mini project: the four pipeline versions, side by side.
//!
//! Run:      cargo bench -p performance --bench pipeline
//! Filter:   cargo bench -p performance --bench pipeline -- v3
//! Baseline: cargo bench -p performance --bench pipeline -- --save-baseline before
//!           (change the code)
//!           cargo bench -p performance --bench pipeline -- --baseline before
//!
//! Criterion's defaults (100 samples, 5 s of measuring per benchmark) are more
//! precise. The settings in `quick()` keep this course's run to a few seconds.

use std::hint::black_box;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use performance::{logs, pipeline};

fn compare_versions(c: &mut Criterion) {
    // Build the input ONCE, outside the measurement: only `analyse` is timed.
    let log = logs::generate(20_000, 42);

    let mut group = c.benchmark_group("pipeline");
    // Report MB/s as well as time per run.
    group.throughput(Throughput::Bytes(log.len() as u64));
    for (name, analyse) in pipeline::VERSIONS {
        group.bench_with_input(BenchmarkId::from_parameter(name), &log, |b, log| {
            // black_box: "pretend you don't know what this is", so the
            // optimiser can't precompute anything about the input.
            b.iter(|| analyse(black_box(log)))
        });
    }
    group.finish();
}

fn quick() -> Criterion {
    Criterion::default()
        .sample_size(10) // the minimum criterion allows
        .warm_up_time(Duration::from_millis(300))
        .measurement_time(Duration::from_secs(1))
}

criterion_group! {
    name = benches;
    config = quick();
    targets = compare_versions
}
criterion_main!(benches);
