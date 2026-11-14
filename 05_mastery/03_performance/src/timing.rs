//! A tiny stopwatch for programs and examples. For real measurements use
//! criterion (`benches/`), which also warms up, picks how many iterations to
//! run, and tells you whether a change is real or just noise.

use std::hint::black_box;
use std::time::{Duration, Instant};

/// Runs `f` `runs` times and returns the MEDIAN time.
///
/// The median ignores the odd slow run (the OS doing something else, a cold
/// cache on the first try) that would drag an average up. The result of `f`
/// goes through [`black_box`], so the optimiser can't delete the work just
/// because nobody looks at the answer.
///
/// # Panics
///
/// Panics if `runs` is 0.
pub fn median_time<R>(runs: usize, mut f: impl FnMut() -> R) -> Duration {
    assert!(runs > 0, "need at least one run");
    let mut times: Vec<Duration> = (0..runs)
        .map(|_| {
            let start = Instant::now();
            black_box(f());
            start.elapsed()
        })
        .collect();
    times.sort_unstable();
    times[runs / 2]
}

/// "release" or "debug", with a warning for debug builds.
pub fn build_kind() -> &'static str {
    if cfg!(debug_assertions) {
        "debug build: timings are NOT representative, add --release"
    } else {
        "release build"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_runs_the_closure_the_right_number_of_times() {
        let mut calls = 0;
        let _ = median_time(5, || calls += 1);
        assert_eq!(calls, 5);
    }

    #[test]
    #[should_panic(expected = "at least one run")]
    fn zero_runs_is_a_bug() {
        median_time(0, || ());
    }
}
