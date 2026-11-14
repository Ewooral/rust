//! Exercise 3: make it compile (E0425) with AND without a feature.
//!
//! `to_json` only exists when this package's `json` feature is on: look at
//! the `#[cfg]` above it. `main` calls it anyway, so the default build fails.
//!
//! Fix `main` only (leave `to_json` alone) so that:
//! - without the feature it prints the plain line, and
//! - with the feature it prints the JSON line too.
//!
//! You might try `if cfg!(feature = "json") { ... }` first. Why does that
//! still fail? (Hint: `cfg!` is a true/false value, so both branches are
//! compiled. `#[cfg]` removes code before it is compiled.)
//!
//! Run:  cargo run -p cargo_and_crates_io --features exercises --bin ex03_fix_feature_gate
//! Then: cargo run -p cargo_and_crates_io --features exercises,json --bin ex03_fix_feature_gate

struct Score {
    name: &'static str,
    points: u32,
}

fn to_plain(score: &Score) -> String {
    format!("{}: {} points", score.name, score.points)
}

#[cfg(feature = "json")]
fn to_json(score: &Score) -> String {
    format!(r#"{{"name":"{}","points":{}}}"#, score.name, score.points)
}

fn main() {
    let score = Score {
        name: "Ama",
        points: 42,
    };
    println!("{}", to_plain(&score));
    println!("{}", to_json(&score));
}
