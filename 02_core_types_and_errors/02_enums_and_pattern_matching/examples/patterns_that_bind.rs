//! Patterns that bind: pull the data out of a variant while matching it.
//! Run: cargo run -p enums_and_pattern_matching --example patterns_that_bind

#[derive(Debug, Clone, Copy)]
enum UsState {
    Alabama,
    Alaska,
}

impl UsState {
    fn existed_in(&self, year: u16) -> bool {
        match self {
            UsState::Alabama => year >= 1819,
            UsState::Alaska => year >= 1959,
        }
    }
}

#[derive(Debug)]
enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter(UsState), // quarters carry the state on their back
}

fn value_in_cents(coin: &Coin) -> u8 {
    match coin {
        Coin::Penny => 1,
        Coin::Nickel => 5,
        Coin::Dime => 10,
        // `state` is a new variable bound to the data inside the variant.
        Coin::Quarter(state) => {
            println!("State quarter from {state:?}!");
            25
        }
    }
}

#[derive(Debug)]
enum Message {
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(u8, u8, u8),
}

fn describe(msg: &Message) -> String {
    match msg {
        // Struct-like variants: name the fields you want.
        Message::Move { x, y } => format!("move to ({x}, {y})"),
        // Matching on a reference binds references: `text` is a `&String`.
        // Nothing is moved out of `msg`.
        Message::Write(text) => format!("write {text:?}"),
        Message::ChangeColor(r, g, b) => format!("colour #{r:02x}{g:02x}{b:02x}"),
    }
}

fn main() {
    let coins = [
        Coin::Penny,
        Coin::Nickel,
        Coin::Dime,
        Coin::Quarter(UsState::Alaska),
        Coin::Quarter(UsState::Alabama),
    ];
    let total: u32 = coins.iter().map(|c| u32::from(value_in_cents(c))).sum();
    println!("total: {total}c");

    if let Coin::Quarter(state) = &coins[3] {
        println!("{state:?} existed in 1900? {}", state.existed_in(1900));
    }

    let messages = [
        Message::Move { x: 1, y: 2 },
        Message::Write(String::from("hi")),
        Message::ChangeColor(255, 128, 0),
    ];
    for m in &messages {
        println!("{}", describe(m));
    }
}
