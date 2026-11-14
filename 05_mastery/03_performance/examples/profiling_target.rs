//! A program made to be profiled: one function eats most of the time.
//!
//! Run:  cargo run -p performance --example profiling_target --release
//!
//! Then point a profiler at the built program (README section 4):
//!   Linux:   perf record -g target/release/examples/profiling_target
//!            perf report
//!   Windows: record target\release\examples\profiling_target.exe with
//!            Windows Performance Recorder (open the trace in Windows
//!            Performance Analyzer), Superluminal, or Visual Studio's
//!            CPU Usage tool.
//! For readable function names, build with debug info (README section 2).

use std::hint::black_box;
use std::time::Instant;

use performance::{logs, pipeline};

fn main() {
    let start = Instant::now();
    // Small in a debug build so it always ends quickly, but big enough in a
    // release build for a sampling profiler to collect plenty of samples.
    let lines = if cfg!(debug_assertions) {
        10_000
    } else {
        300_000
    };
    let log = logs::generate(lines, 7);

    let mut total = 0;
    for _ in 0..3 {
        // The naive version: a profile shows its time going to to_string,
        // the allocator (alloc and free) and HashMap hashing.
        let report = pipeline::analyse_naive(black_box(&log));
        total += report.lines;
    }
    println!(
        "analysed {total} lines in {:.2?}; now profile me (see the header)",
        start.elapsed()
    );
}
