//! Testing randomness: inject the random generator, and seed it in tests.
//!
//! Run:  cargo run -p advanced_testing --example seeded_randomness
//! Test: cargo test -p advanced_testing --example seeded_randomness

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

/// Rolls `n` six-sided dice. The generator is a PARAMETER: production
/// passes a truly random one, tests pass a seeded one.
fn roll_dice(rng: &mut impl Rng, n: usize) -> Vec<u8> {
    (0..n).map(|_| rng.random_range(1..=6)).collect()
}

/// A shuffled deck of 52 cards, as (rank 1..=13, suit 0..4).
fn shuffled_deck(rng: &mut impl Rng) -> Vec<(u8, u8)> {
    let mut deck: Vec<(u8, u8)> = (0..4)
        .flat_map(|suit| (1..=13).map(move |rank| (rank, suit)))
        .collect();
    deck.shuffle(rng);
    deck
}

fn main() {
    let mut random = StdRng::from_os_rng();
    println!(
        "from the OS:  {:?}  (different every run)",
        roll_dice(&mut random, 8)
    );
    let mut seeded = StdRng::seed_from_u64(42);
    println!(
        "seed 42:      {:?}  (the same every run)",
        roll_dice(&mut seeded, 8)
    );
    let top: Vec<(u8, u8)> = shuffled_deck(&mut seeded).into_iter().take(3).collect();
    println!("then three cards off a shuffled deck: {top:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_gives_the_same_rolls() {
        let a = roll_dice(&mut StdRng::seed_from_u64(7), 100);
        let b = roll_dice(&mut StdRng::seed_from_u64(7), 100);
        assert_eq!(a, b);
    }

    // Assert PROPERTIES, not exact values: StdRng's algorithm may change in a
    // future rand version (its docs say so). Pin exact values only with a
    // portable generator such as the rand_chacha crate.
    #[test]
    fn dice_are_between_one_and_six() {
        let rolls = roll_dice(&mut StdRng::seed_from_u64(1), 10_000);
        assert!(rolls.iter().all(|r| (1..=6).contains(r)));
    }

    #[test]
    fn every_face_turns_up_about_a_sixth_of_the_time() {
        // Deterministic thanks to the seed: this can't flake.
        let rolls = roll_dice(&mut StdRng::seed_from_u64(2), 60_000);
        for face in 1..=6 {
            let count = rolls.iter().filter(|&&r| r == face).count();
            assert!((9_000..11_000).contains(&count), "face {face}: {count}");
        }
    }

    #[test]
    fn a_shuffled_deck_has_every_card_once() {
        for seed in 0..20 {
            let mut deck = shuffled_deck(&mut StdRng::seed_from_u64(seed));
            deck.sort_unstable();
            deck.dedup();
            assert_eq!(deck.len(), 52, "seed {seed}");
        }
    }

    #[test]
    fn different_seeds_shuffle_differently() {
        let a = shuffled_deck(&mut StdRng::seed_from_u64(1));
        let b = shuffled_deck(&mut StdRng::seed_from_u64(2));
        assert_ne!(a, b);
    }
}
