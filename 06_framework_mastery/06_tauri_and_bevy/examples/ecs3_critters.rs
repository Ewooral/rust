//! ECS part 3: the critters simulation, frame by frame.
//! Run: cargo run -p tauri_and_bevy --example ecs3_critters
//!      cargo run -p tauri_and_bevy --example ecs3_critters -- --watch   (animated, in a terminal)
//!
//! A different seed gives a different field: -- --seed 7

use std::env;
use std::thread;
use std::time::Duration;

use tauri_and_bevy::critters::{self, Log};

fn main() {
    let args: Vec<String> = env::args().collect();
    let watch = args.iter().any(|a| a == "--watch");
    let seed = args
        .iter()
        .position(|a| a == "--seed")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);

    let mut world = critters::new_world(seed);
    let schedule = critters::schedule();

    for tick in 0..=24 {
        if watch {
            // ESC[2J clears the screen, ESC[H moves to the top left.
            print!("\x1b[2J\x1b[H{}", critters::render(&world));
            thread::sleep(Duration::from_millis(250));
        } else if tick % 8 == 0 {
            println!("{}", critters::render(&world));
        }
        schedule.run(&mut world);
    }
    println!("log:");
    for line in world.resource::<Log>().0.iter() {
        println!("  {line}");
    }
}
