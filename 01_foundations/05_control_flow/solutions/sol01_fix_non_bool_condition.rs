//! Solution to exercise 1: make it compile.
//!
//! A comparison like `> 0` produces a `bool`, which is what `if` needs. It
//! also reads exactly like the sentence: "if there are more than 0 unread".
//!
//! Run: cargo run -p control_flow --features solutions --bin sol01_fix_non_bool_condition

fn main() {
    let unread_messages = 3;

    if unread_messages > 0 {
        println!("You have {unread_messages} unread messages");
    } else {
        println!("Inbox zero!");
    }
}
