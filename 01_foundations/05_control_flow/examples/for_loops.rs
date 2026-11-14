//! `for` runs once for each item in a collection or range: the loop you'll
//! use most. The range helpers at the end are Beyond the Book.
//! Run: cargo run -p control_flow --example for_loops

fn main() {
    // Over an array: no index, so no way to go out of bounds.
    let a = [10, 20, 30, 40, 50];
    for element in a {
        println!("the value is: {element}");
    }

    // Over a range. `1..4` stops BEFORE 4; `1..=4` includes it.
    for number in 1..4 {
        print!("{number} ");
    }
    println!("(1..4)");
    for number in 1..=4 {
        print!("{number} ");
    }
    println!("(1..=4)");

    // The Book's countdown, with `.rev()` to reverse the range.
    for number in (1..4).rev() {
        println!("{number}!");
    }
    println!("LIFTOFF!!!");

    // Beyond the Book: `.step_by(n)` takes every nth number.
    for minute in (0..60).step_by(15) {
        print!(":{minute:02} ");
    }
    println!("(every 15 minutes)");

    // Beyond the Book: `.iter().enumerate()` gives (position, item) pairs, so
    // you get the index too without managing it yourself.
    let podium = ["Ama", "Kofi", "Esi"];
    for (place, name) in podium.iter().enumerate() {
        println!("{}. {name}", place + 1);
    }

    // `_` when you need the repetition but not the number.
    for _ in 0..3 {
        print!("hip ");
    }
    println!("hooray!");
}
