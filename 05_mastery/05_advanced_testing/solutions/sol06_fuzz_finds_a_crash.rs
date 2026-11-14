//! Solution to exercise 6: make the tests pass. The fuzz harness crashed the parser.
//!
//! `parse_price` reads prices such as "12.50" or "3" into pence. Its
//! example-based test passes. The fuzz test feeds it 10 000 random strings
//! (built from seeded random bytes with the `arbitrary` crate, exactly what
//! a `cargo fuzz` target would receive) and checks two things:
//!   1. it never panics, whatever the input;
//!   2. a price it accepts prints and parses back to the same value.
//!
//! Run the tests and read the panic. Fix `parse_price` so it returns `None`
//! for bad input instead of crashing, then run again: the seed corpus has
//! another nasty input waiting. Hints: `str::get(range)` returns an Option
//! instead of panicking; `checked_mul` and `checked_add` return None on
//! overflow; pence must be exactly two ASCII digits.
//!
//! Test: cargo test -p advanced_testing --features solutions --bin sol06_fuzz_finds_a_crash

/// "12.50" -> Some(1250), "3" -> Some(300), anything else -> None.
fn parse_price(text: &str) -> Option<u64> {
    let (pounds, pence) = match text.split_once('.') {
        // Crash 1 was `&text[dot + 1..dot + 3]` on "." or "1.5": slicing past
        // the end panics. Now the pence must be exactly two ASCII digits.
        Some((pounds, pence)) if pence.len() == 2 && pence.bytes().all(|b| b.is_ascii_digit()) => {
            (pounds, pence)
        }
        Some(_) => return None,
        None => (text, "00"),
    };
    // `parse` alone would also accept "+5"; insist on plain digits.
    if pounds.is_empty() || !pounds.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let pounds: u64 = pounds.parse().ok()?;
    let pence: u64 = pence.parse().ok()?;
    // Crash 2 was `pounds * 100` overflowing for huge numbers.
    pounds.checked_mul(100)?.checked_add(pence)
}

fn format_price(pence: u64) -> String {
    format!("{}.{:02}", pence / 100, pence % 100)
}

fn main() {
    for text in ["12.50", "3", "abc"] {
        let parsed = parse_price(text);
        println!("{text:>6} -> {parsed:?} -> {:?}", parsed.map(format_price));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arbitrary::{Arbitrary, Unstructured};
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    /// The harness: `fuzz_target!(|text: &str| ...)` builds its input the
    /// same way, with `arbitrary_take_rest` (all the bytes, as text).
    fn fuzz_one(data: &[u8]) {
        let Ok(text) = <&str>::arbitrary_take_rest(Unstructured::new(data)) else {
            return; // not valid UTF-8: not a string at all
        };
        if let Some(pence) = parse_price(text) {
            assert_eq!(parse_price(&format_price(pence)), Some(pence), "{text:?}");
        }
    }

    #[test]
    fn examples_work() {
        assert_eq!(parse_price("12.50"), Some(1_250));
        assert_eq!(parse_price("3"), Some(300));
        assert_eq!(parse_price("abc"), None);
    }

    #[test]
    fn fuzz_seed_corpus() {
        // Inputs that broke parsers before. Fuzzers keep these forever.
        for seed in [
            "",
            ".",
            "0.00",
            "18446744073709551615",
            "184467440737095517",
        ] {
            fuzz_one(seed.as_bytes());
        }
    }

    #[test]
    fn fuzz_ten_thousand_random_inputs() {
        let mut rng = StdRng::seed_from_u64(6);
        for _ in 0..10_000 {
            let len = rng.random_range(0..12);
            // Mostly digits and dots, so the parser gets past its first line.
            let data: Vec<u8> = (0..len)
                .map(|_| b"0123456789.."[rng.random_range(0..12)])
                .collect();
            fuzz_one(&data);
        }
    }
}
