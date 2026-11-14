//! Solution to exercise 1: make it compile (E0005).
//!
//! `let` needs a pattern that ALWAYS matches (an irrefutable one). But
//! `Some(price)` fails when the item isn't on the menu, so the compiler
//! refuses it. Rewrite the marked line with `let ... else`, so that an
//! unknown item prints "sorry, no <item> today" and returns early.
//! Leave the rest of `order` unchanged and un-indented.
//!
//! Run: cargo run -p patterns_and_matching --features solutions --bin sol01_fix_refutable_let

fn price_in_pence(item: &str) -> Option<u32> {
    match item {
        "tea" => Some(150),
        "cake" => Some(325),
        "toast" => Some(90),
        _ => None,
    }
}

fn order(item: &str) {
    // let ... else: bind `price` if it's Some, otherwise the else block runs
    // and MUST leave the function (return, break, continue or panic).
    let Some(price) = price_in_pence(item) else {
        println!("sorry, no {item} today");
        return;
    };

    let pounds = price / 100;
    let pence = price % 100;
    println!("{item}: £{pounds}.{pence:02}");
}

fn main() {
    order("tea");
    order("pizza");
    order("cake");
}
