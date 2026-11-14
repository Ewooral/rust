//! The mini project on the PC: the same Game of Life core the browser uses,
//! drawn in the terminal.
//!
//! ```text
//! cargo run -p webassembly                 # a glider, a pulsar's period, a random soup
//! cargo run -p webassembly -- toad 4       # one built-in pattern, 4 generations
//! ```
//!
//! For the browser version, see the README: `wasm-pack build --target web`,
//! then open `www/index.html` (not run here).

use webassembly::host::{Host, StdHost};
use webassembly::{Universe, pattern};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first() {
        Some(name) => {
            let generations = args.get(1).and_then(|n| n.parse().ok()).unwrap_or(4);
            show_pattern(name, generations);
        }
        None => {
            show_pattern("glider", 4);
            find_period("pulsar");
            random_soup();
        }
    }
}

/// Prints a pattern's first few generations, side by side.
fn show_pattern(name: &str, generations: usize) {
    let Some(pattern) = pattern::builtin(name) else {
        let names: Vec<&str> = pattern::builtin_names().collect();
        eprintln!("no pattern called {name:?}; try one of {names:?}");
        return;
    };
    let mut universe = Universe::new(pattern.width + 4, pattern.height + 4);
    universe.insert(&pattern, 1, 1);
    let mut frames = vec![universe.to_string()];
    for _ in 0..generations {
        universe.tick();
        frames.push(universe.to_string());
    }
    println!("{} ({name}), generations 0 to {generations}:", pattern.name);
    let rows: Vec<Vec<&str>> = frames.iter().map(|f| f.lines().collect()).collect();
    for line in 0..rows[0].len() {
        let parts: Vec<&str> = rows.iter().map(|frame| frame[line]).collect();
        println!("  {}", parts.join("   "));
    }
    println!();
}

/// Ticks until the grid repeats: an oscillator's period.
fn find_period(name: &str) {
    let pattern = pattern::builtin(name).expect("built-in");
    let mut universe = Universe::new(pattern.width + 4, pattern.height + 4);
    universe.insert(&pattern, 2, 2);
    let start = universe.cells().to_vec();
    for generation in 1..=100 {
        universe.tick();
        if universe.cells() == start {
            println!(
                "{} repeats every {generation} generations ({} cells alive)\n",
                pattern.name,
                universe.population()
            );
            return;
        }
    }
    println!("{} did not repeat within 100 generations\n", pattern.name);
}

/// A random 32x16 grid, filled with the same seed every run.
fn random_soup() {
    let mut host = StdHost::with_seed(2026);
    let mut universe = Universe::new(32, 16);
    universe.randomize(&mut host, 0.35);
    host.log("random soup: 32x16, 35% alive");
    print!("population by generation:");
    for generation in 0..=100 {
        if generation % 20 == 0 {
            print!("  {generation}: {}", universe.population());
        }
        universe.tick();
    }
    println!("\n\n{universe}");
}
