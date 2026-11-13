//! `match`: compare a value against patterns and run the first arm that fits.
//! Run: cargo run -p enums_and_pattern_matching --example match_basics

#[derive(Debug, Clone, Copy)]
enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter,
}

fn value_in_cents(coin: Coin) -> u8 {
    // Each arm is `pattern => expression`. The value of the chosen arm is
    // the value of the whole match.
    match coin {
        // An arm can be a block when it needs several lines.
        Coin::Penny => {
            println!("Lucky penny!");
            1
        }
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter => 25,
    }
}

fn main() {
    let purse = [Coin::Penny, Coin::Dime, Coin::Quarter, Coin::Nickel];
    let mut total = 0;
    for coin in purse {
        let cents = value_in_cents(coin);
        println!("{coin:?} = {cents}c");
        total += u32::from(cents);
    }
    println!("total: {total}c");

    // match works on any type, not only enums. Arms are tried top to bottom.
    let n = 7;
    let size = match n {
        0 => "none",
        1 | 2 => "a couple",  // `|` means "or"
        3..=9 => "a handful", // inclusive range
        _ => "lots",          // anything else
    };
    println!("{n} is {size}");
}
