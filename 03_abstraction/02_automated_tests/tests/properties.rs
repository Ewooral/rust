//! Beyond the Book: PROPERTY-BASED tests with `proptest`.
//!
//! Instead of a few hand-picked examples, describe what ANY valid input looks
//! like, plus a property that must ALWAYS hold. proptest generates hundreds
//! of random cases, and when one fails it SHRINKS it to the smallest input
//! that still fails, so the report is easy to read.
//!
//! Run just this file: cargo test -p automated_tests --test properties

use automated_tests::bowling::{BowlingError, score};
use proptest::prelude::*;

/// A strategy that generates one valid frame (not the 10th): a strike, or
/// two rolls that add up to at most 10.
fn frame() -> impl Strategy<Value = Vec<u32>> {
    prop_oneof![
        Just(vec![10]),
        (0u32..10)
            .prop_flat_map(|first| (Just(first), 0..=10 - first))
            .prop_map(|(a, b)| vec![a, b]),
    ]
}

/// Nine ordinary frames followed by a valid 10th frame with its bonus rolls.
fn valid_game() -> impl Strategy<Value = Vec<u32>> {
    let tenth = prop_oneof![
        // open 10th frame
        (0u32..10)
            .prop_flat_map(|a| (Just(a), 0..10 - a))
            .prop_map(|(a, b)| vec![a, b]),
        // spare + 1 bonus roll
        ((0u32..10), (0u32..=10)).prop_map(|(a, bonus)| vec![a, 10 - a, bonus]),
        // strike + 2 bonus rolls that fit the pins
        (0u32..=10)
            .prop_flat_map(|b1| {
                let max_b2 = if b1 == 10 { 10 } else { 10 - b1 };
                (Just(b1), 0..=max_b2)
            })
            .prop_map(|(b1, b2)| vec![10, b1, b2]),
    ];
    (proptest::collection::vec(frame(), 9), tenth)
        .prop_map(|(frames, last)| frames.into_iter().flatten().chain(last).collect())
}

proptest! {
    // Every valid game scores between 0 and 300.
    #[test]
    fn valid_games_score_between_0_and_300(rolls in valid_game()) {
        let total = score(&rolls).expect("a generated game should be valid");
        prop_assert!(total <= 300, "score {} for {:?}", total, rolls);
    }

    // A game's score is never LESS than the pins knocked down: bonuses only add.
    #[test]
    fn score_is_at_least_the_pin_count(rolls in valid_game()) {
        let pins: u32 = rolls.iter().sum();
        prop_assert!(score(&rolls)? >= pins);
    }

    // Any list of numbers at all: the scorer may say Err, but never panics.
    #[test]
    fn never_panics_on_any_input(rolls in proptest::collection::vec(0u32..20, 0..30)) {
        let _ = score(&rolls);
    }

    // Removing the last roll of a valid game always makes it incomplete.
    #[test]
    fn truncated_games_are_incomplete(rolls in valid_game()) {
        let short = &rolls[..rolls.len() - 1];
        prop_assert_eq!(score(short), Err(BowlingError::Incomplete));
    }
}
