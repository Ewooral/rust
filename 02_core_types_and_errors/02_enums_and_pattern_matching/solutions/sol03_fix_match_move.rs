//! Solution to exercise 3: make it compile (E0382), WITHOUT `.clone()`.
//!
//! Matching on `msg` itself moved the String out of it. Change ONE thing so
//! the match only borrows, and `msg` can still be printed afterwards.
//!
//! Run: cargo run -p enums_and_pattern_matching --features solutions --bin sol03_fix_match_move

#[derive(Debug)]
enum Message {
    Quit,
    Write(String),
}

fn main() {
    let msg = Message::Write(String::from("hello"));
    match &msg {
        Message::Quit => println!("quit"),
        Message::Write(text) => println!("text has {} bytes", text.len()),
    }
    println!("the message was {msg:?}");
    println!("next message: {:?}", Message::Quit);
}
