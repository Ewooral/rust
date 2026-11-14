//! Exercise 1: make it compile (E0005).
//!
//! `let` needs a pattern that ALWAYS matches (an irrefutable one). But
//! `Some(price)` fails when the item isn't on the menu, so the compiler
//! refuses it. Rewrite the marked line with `let ... else`, so that an
//! unknown item prints "sorry, no <item> today" and returns early.
//! Leave the rest of `order` unchanged and un-indented.
//!
//! Run: cargo run -p patterns_and_matching --features exercises --bin ex01_fix_refutable_let

fn price_in_pence(item: &str) -> Option<u32> {
    match item {
        "tea" => Some(150),
        "cake" => Some(325),
        "toast" => Some(90),
        _ => None,
    }
}

fn order(item: &str) {
    let Some(price) = price_in_pence(item); // <- fix this line

    let pounds = price / 100;
    let pence = price % 100;
    println!("{item}: £{pounds}.{pence:02}");
}

fn main() {
    order("tea");
    order("pizza");
    order("cake");
}
