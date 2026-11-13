//! Exercise 1: make it compile.
//!
//! The player levels up, but the program does not compile (E0594).
//! Fix it with ONE small change. Then answer: can you make only the `level`
//! field mutable and leave `name` locked? (No. Why not?)
//!
//! Run: cargo run -p structs --features exercises --bin ex01_fix_mutability

#[derive(Debug)]
struct Player {
    name: String,
    level: u32,
}

fn main() {
    let player = Player {
        name: String::from("Ferris"),
        level: 1,
    };
    player.level += 1;
    println!("{} reached level {}", player.name, player.level);
}
