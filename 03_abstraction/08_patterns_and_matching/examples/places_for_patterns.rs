//! Book 19.1: all the places patterns can be used.
//! Run: cargo run -p patterns_and_matching --example places_for_patterns
//!
//! A pattern is a shape that a value is compared against, and that can pull
//! the value apart into named pieces. You have been using patterns since
//! lesson 01: every `let x = ...` is one.

fn main() {
    // ---------- 1. match arms ----------
    // `match VALUE { PATTERN => EXPRESSION, ... }`. Must be exhaustive.
    let maybe_age: Option<u8> = Some(9);
    let text = match maybe_age {
        None => String::from("unknown age"),
        Some(age) => format!("{age} years old"),
    };
    println!("match:          {text}");

    // ---------- 2. let statements ----------
    // `let PATTERN = EXPRESSION;`. A tuple pattern takes a tuple apart.
    let (x, y, z) = (1, 2, 3);
    println!("let:            x={x} y={y} z={z}");
    // let (a, b) = (1, 2, 3);
    //   // E0308: mismatched types, expected a tuple with 3 elements, found one with 2 elements

    // ---------- 3. if let, else if, else if let ----------
    // Mix them freely. Each condition can test something unrelated.
    let favorite_color: Option<&str> = None;
    let is_tuesday = false;
    let age: Result<u8, _> = "34".parse();

    if let Some(color) = favorite_color {
        println!("if let:         using your favorite color, {color}");
    } else if is_tuesday {
        println!("if let:         Tuesday is green day!");
    } else if let Ok(age) = age {
        // this `age` is a NEW variable that shadows the Result, from here to `}`
        if age > 30 {
            println!("if let:         using purple as the background color");
        } else {
            println!("if let:         using orange as the background color");
        }
    } else {
        println!("if let:         using blue as the background color");
    }

    // ---------- 4. while let ----------
    // Loop for as long as the pattern keeps matching.
    let mut stack = vec![1, 2, 3];
    print!("while let:      ");
    while let Some(top) = stack.pop() {
        print!("{top} ");
    }
    println!();

    // ---------- 5. for loops ----------
    // The thing right after `for` is a pattern. enumerate() gives (index, value).
    let letters = ['a', 'b', 'c'];
    print!("for:            ");
    for (index, letter) in letters.iter().enumerate() {
        print!("{letter} is at {index}; ");
    }
    println!();

    // ---------- 6. function parameters ----------
    let point = (3, 5);
    print_coordinates(&point);

    // ---------- 7. closure parameters ----------
    let pairs = [(1, 'x'), (2, 'y')];
    let numbers: Vec<i32> = pairs.iter().map(|&(n, _letter)| n * 10).collect();
    println!("closure:        {numbers:?}");

    // ---------- 8. let ... else ----------
    // Bind if it matches; otherwise the else block must leave (return, break,
    // continue, panic). See lesson 02/02.
    println!("let-else:       {:?}", parse_age("34"));
    println!("let-else:       {:?}", parse_age("old"));
}

/// The parameter itself is a pattern: `&(x, y)` takes the reference to a
/// tuple apart into two plain integers.
fn print_coordinates(&(x, y): &(i32, i32)) {
    println!("fn parameter:   current location: ({x}, {y})");
}

fn parse_age(text: &str) -> Result<u8, String> {
    let Ok(age) = text.trim().parse::<u8>() else {
        return Err(format!("{text:?} is not an age")); // must diverge
    };
    // from here on, `age` is a plain u8: no nesting
    Ok(age)
}
