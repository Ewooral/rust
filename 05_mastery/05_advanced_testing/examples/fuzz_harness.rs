//! Fuzzing with `arbitrary`: turn random bytes into structured inputs.
//!
//! Run:  cargo run -p advanced_testing --example fuzz_harness
//! Test: cargo test -p advanced_testing --example fuzz_harness
//!
//! A real fuzzer (cargo-fuzz, see the README) calls `fuzz_one` with bytes it
//! evolves to reach new code. Here a seeded generator supplies the bytes, so
//! the same harness runs as an ordinary, repeatable test.

use std::collections::HashMap;

use arbitrary::{Arbitrary, Unstructured};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// The code under test: a stock counter that must never go negative.
#[derive(Default)]
struct Stock {
    items: HashMap<String, u32>,
}

impl Stock {
    fn add(&mut self, item: &str, n: u32) {
        let count = self.items.entry(item.to_string()).or_insert(0);
        *count = count.saturating_add(n);
    }

    /// Takes `n` if there are enough; otherwise takes nothing.
    fn take(&mut self, item: &str, n: u32) -> bool {
        // The first run of this harness failed with [Take { item: 109, n: 0 }]:
        // taking 0 of an item never stocked said `false`, the model said
        // `true`. Nobody had decided that case. Now it's decided: taking
        // nothing always works.
        if n == 0 {
            return true;
        }
        match self.items.get_mut(item) {
            Some(count) if *count >= n => {
                *count -= n;
                true
            }
            _ => false,
        }
    }

    fn count(&self, item: &str) -> u32 {
        self.items.get(item).copied().unwrap_or(0)
    }
}

/// Fuzzer bytes become a list of these. `derive(Arbitrary)` writes the code
/// that turns bytes into commands.
#[derive(Debug, Arbitrary)]
enum Command {
    Add { item: u8, n: u32 },
    Take { item: u8, n: u32 },
}

/// The harness: run the commands on the real `Stock` and on a dead-simple
/// MODEL of it (a plain array), and check that they always agree. Must not
/// panic for any input.
fn fuzz_one(data: &[u8]) {
    // `arbitrary_take_rest` uses ALL the bytes: exactly what cargo-fuzz's
    // `fuzz_target!(|commands: Vec<Command>| ...)` does with its input.
    let Ok(commands) = Vec::<Command>::arbitrary_take_rest(Unstructured::new(data)) else {
        return;
    };
    let mut stock = Stock::default();
    let mut model = [0u64; 4]; // only 4 item names, so they collide often
    for command in &commands {
        match *command {
            Command::Add { item, n } => {
                let i = usize::from(item % 4);
                stock.add(&format!("item{i}"), n);
                model[i] = (model[i] + u64::from(n)).min(u64::from(u32::MAX));
            }
            Command::Take { item, n } => {
                let i = usize::from(item % 4);
                let took = stock.take(&format!("item{i}"), n);
                assert_eq!(took, model[i] >= u64::from(n), "{commands:?}");
                if took {
                    model[i] -= u64::from(n);
                }
            }
        }
    }
    for (i, &expected) in model.iter().enumerate() {
        assert_eq!(
            u64::from(stock.count(&format!("item{i}"))),
            expected,
            "{commands:?}"
        );
    }
}

/// Seeded random byte strings, standing in for a fuzzer.
fn run_random_inputs(seed: u64, inputs: usize) {
    let mut rng = StdRng::seed_from_u64(seed);
    for _ in 0..inputs {
        let len = rng.random_range(0..200);
        let data: Vec<u8> = (0..len).map(|_| rng.random()).collect();
        fuzz_one(&data);
    }
}

fn main() {
    // What fuzzer bytes turn into: the first of 100 random 24-byte inputs
    // that decodes to two or more commands. Before each element of a Vec,
    // arbitrary reads one "keep going?" byte (odd means yes); a command then
    // takes 4 bytes to pick the variant, 1 for `item` and 4 for `n`.
    let mut rng = StdRng::seed_from_u64(1);
    let found = (0..100).find_map(|_| {
        let bytes: Vec<u8> = (0..24).map(|_| rng.random()).collect();
        let commands = Vec::<Command>::arbitrary_take_rest(Unstructured::new(&bytes)).ok()?;
        (commands.len() >= 2).then_some((bytes, commands))
    });
    if let Some((bytes, commands)) = found {
        println!("{bytes:?}\n  -> {commands:?}");
    }
    run_random_inputs(1, 5_000);
    println!("5 000 random inputs: no crash, Stock always agreed with the model");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn survives_empty_and_tiny_inputs() {
        for data in [&[][..], &[0], &[255; 3]] {
            fuzz_one(data);
        }
    }

    #[test]
    fn survives_twenty_thousand_random_inputs() {
        run_random_inputs(0xF022, 20_000);
    }
}
