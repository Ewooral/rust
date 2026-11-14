//! Allocate only when you must: `Cow` and the small-vector idea.
//!
//! Run: cargo run -p performance --example cow_and_smallvec

use std::borrow::Cow;

use performance::alloc_counter::{CountingAlloc, count_allocations};
use smallvec::SmallVec;

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

/// Always returns a new String, even when nothing needed changing.
fn tabs_to_spaces_always(line: &str) -> String {
    line.replace('\t', "    ")
}

/// Borrows the line when it has no tabs (most lines), and only allocates a
/// new String for the few that need changing: "clone on write".
fn tabs_to_spaces_cow(line: &str) -> Cow<'_, str> {
    if line.contains('\t') {
        Cow::Owned(line.replace('\t', "    "))
    } else {
        Cow::Borrowed(line)
    }
}

/// Positions of the commas in `line`. Most lines have a handful, so the
/// first 8 live inline, inside the SmallVec itself; only a line with more
/// than 8 "spills" onto the heap.
fn comma_positions(line: &str) -> SmallVec<[usize; 8]> {
    line.match_indices(',').map(|(i, _)| i).collect()
}

fn main() {
    // 1 000 lines, 1 in 100 with a tab.
    let lines: Vec<String> = (0..1_000)
        .map(|i| {
            if i % 100 == 0 {
                format!("\tindented line {i}")
            } else {
                format!("plain line {i}")
            }
        })
        .collect();

    let (always, a) = count_allocations(|| {
        lines
            .iter()
            .map(|l| tabs_to_spaces_always(l).len())
            .sum::<usize>()
    });
    let (cow, c) = count_allocations(|| {
        lines
            .iter()
            .map(|l| tabs_to_spaces_cow(l).len())
            .sum::<usize>()
    });
    assert_eq!(always, cow, "same result either way");
    println!("Cow<str>: normalise 1 000 lines (10 contain a tab)");
    println!(
        "  always a new String:        {:>5} allocations",
        a.allocations
    );
    println!(
        "  Cow, borrow when unchanged: {:>5} allocations",
        c.allocations
    );
    match tabs_to_spaces_cow("no tabs here") {
        Cow::Borrowed(_) => println!("  \"no tabs here\" came back Borrowed"),
        Cow::Owned(_) => println!("  \"no tabs here\" came back Owned"),
    }

    println!("\nSmallVec<[usize; 8]>: comma positions");
    for line in ["a,b,c", "1,2,3,4,5,6,7,8,9,10"] {
        let (commas, stats) = count_allocations(|| comma_positions(line));
        println!(
            "  {line:<22} {} commas  spilled: {:<5} allocations: {}",
            commas.len(),
            commas.spilled(),
            stats.allocations
        );
    }
    let (_, vec_stats) = count_allocations(|| {
        "a,b,c"
            .match_indices(',')
            .map(|(i, _)| i)
            .collect::<Vec<usize>>()
    });
    println!(
        "  \"a,b,c\" into a plain Vec instead: {} allocation",
        vec_stats.allocations
    );
}
