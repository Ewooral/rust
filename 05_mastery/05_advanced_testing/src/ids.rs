//! Order numbers from a random generator that tests can seed.
//!
//! Randomness is a dependency just like time. Production seeds the
//! generator from the operating system; tests seed it with a fixed number,
//! so every run produces the same ids.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Letters and digits that can't be confused when read aloud: no 0/O, 1/I.
const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

/// Makes order ids like `ORD-7KQ2MX`.
#[derive(Debug, Clone)]
pub struct OrderIds {
    rng: StdRng,
}

impl OrderIds {
    /// Unpredictable ids, for production.
    pub fn from_os() -> OrderIds {
        OrderIds {
            rng: StdRng::from_os_rng(),
        }
    }

    /// The same sequence of ids every time, for tests.
    pub fn seeded(seed: u64) -> OrderIds {
        OrderIds {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// The next id.
    pub fn next_id(&mut self) -> String {
        let code: String = (0..6)
            .map(|_| ALPHABET[self.rng.random_range(0..ALPHABET.len())] as char)
            .collect();
        format!("ORD-{code}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_gives_the_same_ids() {
        let mut a = OrderIds::seeded(42);
        let mut b = OrderIds::seeded(42);
        for _ in 0..100 {
            assert_eq!(a.next_id(), b.next_id());
        }
    }

    #[test]
    fn ids_have_the_documented_shape() {
        // Check PROPERTIES of the output, not exact values: StdRng's
        // algorithm may change in a future rand version.
        let mut ids = OrderIds::seeded(7);
        for _ in 0..1_000 {
            let id = ids.next_id();
            assert_eq!(id.len(), 10, "{id}");
            assert!(id.starts_with("ORD-"));
            assert!(id[4..].bytes().all(|b| ALPHABET.contains(&b)), "{id}");
        }
    }

    #[test]
    fn different_seeds_give_different_ids() {
        assert_ne!(OrderIds::seeded(1).next_id(), OrderIds::seeded(2).next_id());
    }
}
