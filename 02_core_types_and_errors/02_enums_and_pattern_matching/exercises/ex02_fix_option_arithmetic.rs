//! Exercise 2: make it compile (E0277).
//!
//! A missing bonus should count as 0. Do NOT use `.unwrap()`: if the bonus is
//! missing, that would crash. Try it with a `match` first, then with a method.
//!
//! Run: cargo run -p enums_and_pattern_matching --features exercises --bin ex02_fix_option_arithmetic

fn bonus_for(name: &str) -> Option<u32> {
    if name == "Ama" { Some(50) } else { None }
}

fn main() {
    for name in ["Ama", "Kofi"] {
        let salary: u32 = 1_000;
        let bonus = bonus_for(name);
        let total = salary + bonus;
        println!("{name} earns {total}");
    }
}
