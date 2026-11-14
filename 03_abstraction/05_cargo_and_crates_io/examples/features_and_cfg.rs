//! Beyond the Book: feature flags, optional dependencies, and `cfg`.
//!
//! Run:  cargo run -p cargo_and_crates_io --example features_and_cfg
//! Then: cargo run -p cargo_and_crates_io --example features_and_cfg --no-default-features
//!       cargo run -p cargo_and_crates_io --example features_and_cfg --features json
//!       cargo run -p cargo_and_crates_io --example features_and_cfg --release
//!
//! The [features] table in Cargo.toml declares switches. Cargo turns each
//! switched-on feature into `--cfg feature="name"` for the compiler, and the
//! code asks about it with `cfg`.

use cargo_and_crates_io::report::MARKS;

// `#[cfg(...)]` REMOVES the item from the build when the condition is false.
// Exactly one of these two functions exists in any build.
#[cfg(windows)]
fn platform() -> &'static str {
    "Windows"
}

#[cfg(not(windows))]
fn platform() -> &'static str {
    "not Windows (Linux, probably)"
}

/// `cfg_attr(condition, attribute)` adds an attribute only when the condition
/// holds. Here: derive `Serialize` only when the json feature brought serde.
#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    // `cfg!(...)` is a true/false VALUE. Both branches are still compiled.
    println!("feature \"unicode\": {}", cfg!(feature = "unicode"));
    println!("feature \"json\":    {}", cfg!(feature = "json"));
    println!("marks in use:      {MARKS:?}");
    println!("debug_assertions:  {}", cfg!(debug_assertions));
    println!(
        "64-bit Windows:    {}",
        cfg!(all(windows, target_pointer_width = "64"))
    );
    println!("platform():        {}", platform());

    let p = Point { x: 3, y: 4 };
    println!("point:             {p:?}, x + y = {}", p.x + p.y);

    // This block only exists with --features json, and only then can it use
    // the optional serde_json dependency.
    #[cfg(feature = "json")]
    {
        let json = serde_json::to_string(&p).expect("a Point always serialises");
        println!("as JSON:           {json}");
    }
    #[cfg(not(feature = "json"))]
    println!("as JSON:           (rebuild with --features json)");
}
