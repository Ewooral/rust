//! Exercise 2: make the tests pass, by allocating once instead of many times.
//!
//! Both functions below give the right answer. The tests also set an
//! ALLOCATION BUDGET: each function may visit the allocator only once.
//! Unlike a timing, an allocation count is exact, so it makes a test that
//! never flakes. Run the tests and read how far over budget each one is.
//!
//! Then fix the functions, keeping their results the same:
//! - `squares`: you know the final length before you start.
//! - `to_csv`: work out an upper bound for the length (a `u32` has at most
//!   10 digits; add the comma and the newline), and stop `id.to_string()`
//!   making a String per row: `write!(out, "{id}")` formats straight into
//!   `out` (it needs `use std::fmt::Write;`).
//!
//! Test: cargo test -p performance --features exercises --bin ex02_preallocate

use performance::alloc_counter::CountingAlloc;

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

/// The squares of 0..n.
fn squares(n: u64) -> Vec<u64> {
    let mut v = Vec::new();
    for i in 0..n {
        v.push(i * i);
    }
    v
}

/// One `id,name` line per row.
fn to_csv(rows: &[(u32, &str)]) -> String {
    let mut out = String::new();
    for (id, name) in rows {
        out.push_str(&id.to_string());
        out.push(',');
        out.push_str(name);
        out.push('\n');
    }
    out
}

fn main() {
    println!("{:?}", squares(6));
    print!(
        "{}",
        to_csv(&[(1, "ada"), (2, "grace"), (4_294_967_295, "max")])
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use performance::alloc_counter::count_allocations;

    #[test]
    fn squares_are_right() {
        assert_eq!(squares(5), [0, 1, 4, 9, 16]);
        assert!(squares(0).is_empty());
    }

    #[test]
    fn squares_allocate_once() {
        let (v, stats) = count_allocations(|| squares(1_000));
        assert_eq!(v.len(), 1_000);
        assert_eq!(stats.allocations, 1, "squares(1_000) is over budget");
    }

    #[test]
    fn csv_is_right() {
        assert_eq!(to_csv(&[(7, "x"), (42, "yz")]), "7,x\n42,yz\n");
        assert_eq!(to_csv(&[]), "");
    }

    #[test]
    fn csv_allocates_once() {
        let names = ["ada", "grace", "alan", "edsger", "barbara"];
        let rows: Vec<(u32, &str)> = (0..100u32)
            .map(|i| (i * 1_000_003, names[i as usize % 5]))
            .collect();
        let (csv, stats) = count_allocations(|| to_csv(&rows));
        assert_eq!(csv.lines().count(), 100);
        assert_eq!(stats.allocations, 1, "to_csv of 100 rows is over budget");
    }
}
