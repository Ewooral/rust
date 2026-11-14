//! Solution to exercise 3: make it compile (E0425) with and without a feature.
//!
//! The call to `to_json` gets the same `#[cfg]` as the function, so it is
//! removed from builds where the function does not exist. `if cfg!(...)`
//! cannot do this: it is an ordinary `if`, and both branches must compile.
//!
//! Run:  cargo run -p cargo_and_crates_io --features solutions --bin sol03_fix_feature_gate
//! Then: cargo run -p cargo_and_crates_io --features solutions,json --bin sol03_fix_feature_gate

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
    #[cfg(feature = "json")]
    println!("{}", to_json(&score));
}
