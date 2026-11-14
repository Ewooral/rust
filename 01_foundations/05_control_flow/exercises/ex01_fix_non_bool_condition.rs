//! Exercise 1: make it compile.
//!
//! In some languages any non-zero number counts as "true". Not in Rust: an
//! `if` condition must be a `bool` (E0308). Rewrite the condition so it says
//! exactly what it means: "if there are any unread messages".
//!
//! Run: cargo run -p control_flow --features exercises --bin ex01_fix_non_bool_condition

fn main() {
    let unread_messages = 3;

    if unread_messages {
        println!("You have {unread_messages} unread messages");
    } else {
        println!("Inbox zero!");
    }
}
