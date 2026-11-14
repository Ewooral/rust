//! Solution to exercise 5: make it compile (E0594 and E0596), the rayon way.
//!
//! Someone made two loops parallel by changing `iter()` to `par_iter()`. In
//! a sequential `for_each`, updating `total` and pushing to `big` is fine.
//! In a parallel one, many threads would do it AT THE SAME TIME: a data
//! race. rayon's `for_each` only accepts an `Fn` closure (one that doesn't
//! change what it captured), so the compiler stops the race before it runs.
//!
//! Don't reach for a Mutex: that makes every thread queue for one lock.
//! Rewrite each function so the threads never share anything:
//! - `total_bytes`: map each file to its size and `sum()` them;
//! - `big_files`: `filter` the big ones, `map` to the name, `collect()`.
//!   (rayon's `collect` keeps the original order.)
//!
//! Test: cargo test -p performance --features solutions --bin sol05_fix_parallel_sum

use rayon::prelude::*;

struct File {
    name: String,
    bytes: u64,
}

// Each thread sums its own share, then rayon adds the partial sums: nothing
// is shared, so there is nothing to lock.
fn total_bytes(files: &[File]) -> u64 {
    files.par_iter().map(|f| f.bytes).sum()
}

// Each thread collects its own share; rayon joins them in the original order.
fn big_files(files: &[File], limit: u64) -> Vec<String> {
    files
        .par_iter()
        .filter(|f| f.bytes > limit)
        .map(|f| f.name.clone())
        .collect()
}

fn sample(n: u64) -> Vec<File> {
    (0..n)
        .map(|i| File {
            name: format!("file{i}.log"),
            bytes: i * 100,
        })
        .collect()
}

fn main() {
    let files = sample(10);
    println!("total: {} bytes", total_bytes(&files));
    println!("bigger than 700: {:?}", big_files(&files, 700));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_is_the_sum_of_all_sizes() {
        assert_eq!(total_bytes(&sample(10)), 4_500);
        assert_eq!(total_bytes(&sample(100_000)), 100 * (99_999 * 100_000 / 2));
    }

    #[test]
    fn big_files_keeps_the_original_order() {
        assert_eq!(big_files(&sample(10), 700), ["file8.log", "file9.log"]);
        let many = big_files(&sample(10_000), 0);
        assert_eq!(many.len(), 9_999);
        assert_eq!(many[0], "file1.log");
        assert_eq!(many[9_998], "file9999.log");
    }
}
