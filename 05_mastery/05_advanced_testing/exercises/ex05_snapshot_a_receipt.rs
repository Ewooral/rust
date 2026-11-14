//! Exercise 5: make the test pass. A snapshot caught a formatting bug.
//!
//! `render` prints a bill. The test compares its WHOLE output with an
//! inline snapshot (the text after `@` in the test), which someone checked
//! by eye when the format was designed. Run the test and read insta's
//! diff: the `-` lines are the snapshot, the `+` lines what the code
//! printed now. Then fix `render` (not the snapshot!).
//!
//! insta also saves the new output in a `.pending-snap` file next to this
//! one (outside CI). That's for `cargo insta review`, which this course
//! doesn't install: delete the file when you're done.
//!
//! Test: cargo test -p advanced_testing --features exercises --bin ex05_snapshot_a_receipt

/// Formats pence as pounds: 705 -> "£7.05".
fn pounds(pence: u64) -> String {
    format!("£{}.{}", pence / 100, pence % 100)
}

/// One line per item, then the total. Amounts are right-aligned to 8.
fn render(items: &[(&str, u64)]) -> String {
    let mut out = String::new();
    for (name, pence) in items {
        out += &format!("{name:<12}{:>8}\n", pounds(*pence));
    }
    let total: u64 = items.iter().map(|(_, p)| p).sum();
    out += &format!("{:<12}{:>8}\n", "TOTAL", pounds(total));
    out
}

fn main() {
    print!("{}", render(&[("Tea", 250), ("Scone", 305), ("Jam", 150)]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bill_for_afternoon_tea() {
        insta::assert_snapshot!(render(&[("Tea", 250), ("Scone", 305), ("Jam", 150)]), @r"
        Tea            £2.50
        Scone          £3.05
        Jam            £1.50
        TOTAL          £7.05
        ");
    }
}
