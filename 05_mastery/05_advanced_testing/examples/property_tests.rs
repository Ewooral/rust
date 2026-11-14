//! Property testing recap (lesson 03/02): strategies, shrinking, configuration.
//!
//! Run:  cargo run -p advanced_testing --example property_tests
//! Test: cargo test -p advanced_testing --example property_tests
//! See shrinking: cargo test -p advanced_testing --example property_tests -- --ignored

use advanced_testing::money::Money;

/// Splits a bill between `people` as fairly as possible: the shares differ
/// by at most a penny and always add up to the bill.
fn split_bill(total: Money, people: u32) -> Vec<Money> {
    let people = u64::from(people.max(1));
    let (share, left_over) = (total.pence() / people, total.pence() % people);
    (0..people)
        .map(|i| Money::from_pence(share + u64::from(i < left_over)))
        .collect()
}

/// A buggy version: it forgets the left-over pence.
fn split_bill_buggy(total: Money, people: u32) -> Vec<Money> {
    let people = u64::from(people.max(1));
    (0..people)
        .map(|_| Money::from_pence(total.pence() / people))
        .collect()
}

fn main() {
    println!(
        "£10.00 between 3: {:?}",
        split_bill(Money::from_pence(1_000), 3)
    );
    println!(
        "buggy version:    {:?}",
        split_bill_buggy(Money::from_pence(1_000), 3)
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use advanced_testing::money::parse_money;
    use proptest::prelude::*;

    // A custom strategy: valid money TEXT, built from parts.
    prop_compose! {
        fn money_text()(pounds in 0u64..1_000_000, pence in 0u64..100, symbol: bool) -> String {
            format!("{}{pounds}.{pence:02}", if symbol { "£" } else { "" })
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        #[test]
        fn shares_add_up_to_the_bill(pence in 0u64..10_000_000, people in 1u32..50) {
            let shares = split_bill(Money::from_pence(pence), people);
            prop_assert_eq!(shares.iter().map(|m| m.pence()).sum::<u64>(), pence);
        }

        // prop_assume! throws a generated case away (it doesn't count).
        #[test]
        fn shares_differ_by_at_most_a_penny(pence in 0u64..10_000_000, people in 0u32..50) {
            prop_assume!(people > 0); // 0 people is a different rule, tested elsewhere
            let shares = split_bill(Money::from_pence(pence), people);
            let max = shares.iter().max().unwrap().pence();
            let min = shares.iter().min().unwrap().pence();
            prop_assert!(max - min <= 1);
        }

        #[test]
        fn valid_money_text_always_parses(text in money_text()) {
            prop_assert!(parse_money(&text).is_ok(), "{} did not parse", text);
        }

        // prop_oneof! picks one of several strategies each time.
        #[test]
        fn bad_endings_never_parse(
            base in money_text(),
            tail in prop_oneof![Just(".".to_string()), Just("0".to_string()), "[a-z]{1,3}"],
        ) {
            let text = format!("{base}{tail}");
            prop_assert!(parse_money(&text).is_err(), "{} parsed", text);
        }
    }

    proptest! {
        // failure_persistence: None stops proptest saving the failing case in
        // a proptest-regressions/ file: this test fails on purpose.
        #![proptest_config(ProptestConfig { failure_persistence: None, ..ProptestConfig::default() })]

        #[test]
        #[ignore = "fails on purpose: run with -- --ignored to watch it shrink"]
        fn buggy_shares_add_up_to_the_bill(pence in 0u64..10_000_000, people in 1u32..50) {
            let shares = split_bill_buggy(Money::from_pence(pence), people);
            prop_assert_eq!(shares.iter().map(|m| m.pence()).sum::<u64>(), pence);
        }
    }
}
