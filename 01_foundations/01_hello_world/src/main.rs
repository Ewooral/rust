//! Mini project for Book chapter 1: a greeting card that knows how it was built.
//!
//! It still says "Hello, world!", but in a box, and it tells you which package
//! and which build profile Cargo used to make it.
//!
//! Run: cargo run -p hello_world               (greets the world)
//! Run: cargo run -p hello_world -- Ama        (greets Ama: everything after `--` goes to us)
//! Run: cargo run -p hello_world --release     (notice the profile change)
//! Test: cargo test -p hello_world

fn main() {
    // `std::env::args()` lists the command-line arguments (Book chapter 12).
    // `.nth(1)` skips argument 0, which is the program's own name, and
    // `unwrap_or` falls back to "world" when no name was given.
    let name = std::env::args().nth(1).unwrap_or(String::from("world"));

    println!("{}", boxed(&greeting(&name)));
    println!("{}", built_by());
}

/// The classic first program, with a name in it: "Hello, Ama!".
fn greeting(name: &str) -> String {
    // `format!` works exactly like `println!`, but gives the text back
    // instead of printing it.
    format!("Hello, {name}!")
}

/// Draws a box around one line of text:
///
/// ```text
/// +---------------+
/// | Hello, world! |
/// +---------------+
/// ```
fn boxed(text: &str) -> String {
    // Count characters, not bytes, so "Héllo" gets the right width.
    let width = text.chars().count() + 2; // one space of padding on each side
    let edge = format!("+{}+", "-".repeat(width));
    format!("{edge}\n| {text} |\n{edge}")
}

/// Which Cargo profile compiled this program.
///
/// `cargo build` and `cargo run` use the **dev** profile, which keeps debug
/// checks on (`debug_assertions`). `--release` turns them off and optimises.
fn build_profile() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}

/// One line about the build. `env!` reads values Cargo sets while compiling,
/// straight from `Cargo.toml`.
fn built_by() -> String {
    format!(
        "built by Cargo: {} v{} ({} build)",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        build_profile()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greets_by_name() {
        assert_eq!(greeting("world"), "Hello, world!");
        assert_eq!(greeting("Ferris"), "Hello, Ferris!");
    }

    #[test]
    fn draws_a_box() {
        assert_eq!(boxed("hi"), "+----+\n| hi |\n+----+");
    }

    #[test]
    fn box_width_counts_characters_not_bytes() {
        // "é" is 2 bytes but 1 character: every line must be equally wide.
        let card = boxed("Héllo");
        let widths: Vec<usize> = card.lines().map(|line| line.chars().count()).collect();
        assert_eq!(widths, [9, 9, 9]);
    }

    #[test]
    fn names_the_package_from_cargo_toml() {
        let line = built_by();
        assert!(line.starts_with("built by Cargo: hello_world v"));
        assert!(line.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn tests_are_debug_builds_by_default() {
        // `cargo test` uses the test profile, which inherits from dev.
        if cfg!(debug_assertions) {
            assert_eq!(build_profile(), "debug");
        } else {
            assert_eq!(build_profile(), "release");
        }
    }
}
