//! Exercise 3: make it compile (E0408).
//!
//! With `|`, the arm's code runs whichever alternative matched, so EVERY
//! alternative must bind the SAME names (with the same types). Here the two
//! sides of each `|` use different names. Fix the patterns so each pair uses
//! one name. Don't split the arms, and don't change the arms' code.
//! Hint for the struct variant: `Tap { x: some_name, .. }` binds field `x`
//! to `some_name`, and `Tap { x, y }` is short for `Tap { x: x, y: y }`.
//!
//! Run: cargo run -p patterns_and_matching --features exercises --bin ex03_fix_or_pattern_bindings

enum Event {
    KeyPress(char),
    KeyRelease(char),
    Click { x: i32, y: i32 },
    Tap { x: i32, y: i32 },
    Scroll(i32),
}

fn describe(event: &Event) -> String {
    match event {
        Event::KeyPress(key) | Event::KeyRelease(k) => format!("key {key:?}"),
        Event::Click { x, y } | Event::Tap { x: left, y } => format!("pointer at ({x}, {y})"),
        Event::Scroll(amount) => format!("scroll by {amount}"),
    }
}

fn main() {
    let events = [
        Event::KeyPress('a'),
        Event::KeyRelease('a'),
        Event::Click { x: 10, y: 20 },
        Event::Tap { x: 3, y: 4 },
        Event::Scroll(-2),
    ];
    for event in &events {
        println!("{}", describe(event));
    }
}
