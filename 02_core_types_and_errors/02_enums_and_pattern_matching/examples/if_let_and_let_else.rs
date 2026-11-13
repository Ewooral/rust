//! `if let` and `let ... else`: concise control flow for one pattern.
//! Run: cargo run -p enums_and_pattern_matching --example if_let_and_let_else

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

#[derive(Debug, Clone, Copy)]
enum Coin {
    Penny,
    Quarter(UsState),
}

fn main() {
    // `if let` = a match with one interesting arm and `_ => ()`.
    let config_max = Some(3u8);
    if let Some(max) = config_max {
        println!("The maximum is configured to be {max}");
    }

    // `if let ... else` = one interesting arm plus "everything else".
    let mut count = 0;
    for coin in [Coin::Penny, Coin::Quarter(UsState::Alaska), Coin::Penny] {
        if let Coin::Quarter(state) = coin {
            println!("State quarter from {state:?}!");
        } else {
            count += 1;
        }
    }
    println!("{count} non-quarter coins");

    for coin in [Coin::Quarter(UsState::Alabama), Coin::Penny] {
        match describe_state_quarter(coin) {
            Some(text) => println!("{text}"),
            None => println!("{coin:?} is not a state quarter"),
        }
    }
}

/// `let ... else`: bind the value if the pattern matches, otherwise the else
/// block MUST leave the function (return, break, continue or panic).
/// It keeps the "happy path" un-indented.
fn describe_state_quarter(coin: Coin) -> Option<String> {
    let Coin::Quarter(state) = coin else {
        return None;
    };

    if state.existed_in(1900) {
        Some(format!("{state:?} is pretty old, for America!"))
    } else {
        Some(format!("{state:?} is relatively new."))
    }
}
