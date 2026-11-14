//! Exercise 3: make it compile (E0277).
//!
//! `#[derive(Settings)]` reads every field with `FromStr` (that's how
//! `"8080"` becomes a `u16`). `LogLevel` doesn't implement `FromStr`, so the
//! generated code doesn't compile. Look at WHERE the error points: at
//! `LogLevel` in the struct, not at the derive. The macro gives the code it
//! generates for each field that field's span (`quote_spanned!`).
//!
//! Fix it by implementing `std::str::FromStr` for `LogLevel`:
//!   - "quiet", "normal" and "verbose" (any letter case) give the three levels;
//!   - anything else is an error. Use `type Err = String;`, a message such
//!     as "expected quiet, normal or verbose". (The derive also needs
//!     `Err: Display`, which `String` is.)
//!
//! Run: cargo run -p macros_in_depth --features exercises --bin ex03_fix_from_str

use macros_in_depth::Settings;

#[derive(Debug, Clone, Copy, PartialEq)]
enum LogLevel {
    Quiet,
    Normal,
    Verbose,
}

#[derive(Settings)]
struct AppSettings {
    /// How much to log: quiet, normal or verbose.
    #[setting(default = "normal")]
    level: LogLevel,
    /// Worker threads.
    #[setting(default = "4")]
    workers: u8,
}

fn main() {
    print!("{}", AppSettings::help());
    for text in ["level = verbose", "workers = 8", "level = LOUD"] {
        match AppSettings::from_text(text) {
            Ok(settings) => println!(
                "{text:?}: {:?} with {} workers",
                settings.level, settings.workers
            ),
            Err(error) => println!("{text:?}: {error}"),
        }
    }
    let _ = (LogLevel::Quiet, LogLevel::Normal); // every level is used
}
