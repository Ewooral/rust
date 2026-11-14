//! A user directory: build a stack of `Arc<dyn UserService>` layers from a
//! preset name, then look users up from many tasks at once, twice.
//!
//! Run:  cargo run -p async_traits_from_scratch
//!       cargo run -p async_traits_from_scratch -- cached 1 2 3
//!       cargo run -p async_traits_from_scratch -- resilient --remote-slow
//!       cargo run -p async_traits_from_scratch -- resilient --remote-down 1 2 3 42
//! Test: cargo test -p async_traits_from_scratch

use std::process::ExitCode;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_traits_from_scratch::{
    InMemoryUsers, PRESETS, RemoteUsers, build_stack, lookup_many, sample_users,
};

const USAGE: &str =
    "usage: async_traits_from_scratch [PRESET] [--remote-down] [--remote-slow] [ID ...]";

#[tokio::main]
async fn main() -> ExitCode {
    let mut preset = String::from("resilient");
    let mut remote_down = false;
    let mut latency = Duration::from_millis(30);
    let mut ids = Vec::new();
    for arg in std::env::args().skip(1) {
        if arg == "--remote-down" {
            remote_down = true;
        } else if arg == "--remote-slow" {
            latency = Duration::from_millis(200);
        } else if let Ok(id) = arg.parse::<u32>() {
            ids.push(id);
        } else if PRESETS.contains(&arg.as_str()) {
            preset = arg;
        } else {
            eprintln!("unknown argument {arg:?}\n{USAGE}\npresets: {PRESETS:?}");
            return ExitCode::FAILURE;
        }
    }
    if ids.is_empty() {
        ids = vec![1, 2, 3];
    }

    // The remote "database" answers in 30 ms (200 ms with --remote-slow).
    // The resilient stack gives it 100 ms, then asks the in-memory copy.
    let remote = Arc::new(RemoteUsers::new(sample_users(), latency));
    remote.set_up(!remote_down);
    let memory = Arc::new(InMemoryUsers::new(sample_users()));
    let service = build_stack(&preset, remote.clone(), memory, Duration::from_millis(100))
        .expect("the preset was checked against PRESETS");

    println!("stack: {}", service.describe());
    for round in 1..=2 {
        let started = Instant::now();
        let answers = lookup_many(service.clone(), &ids).await;
        println!(
            "\nround {round}: {} lookups at once took {} ms",
            ids.len(),
            started.elapsed().as_millis()
        );
        for (id, answer) in ids.iter().zip(answers) {
            match answer {
                Ok(user) => println!("  {id:>3} -> {}", user.name),
                Err(error) => println!("  {id:>3} -> error: {error}"),
            }
        }
    }
    println!("\nthe remote service was asked {} times", remote.calls());
    ExitCode::SUCCESS
}
