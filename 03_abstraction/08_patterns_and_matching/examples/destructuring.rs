//! Book 19.3 (part 2): destructuring structs, enums, nested values, tuples
//! and references.
//! Run: cargo run -p patterns_and_matching --example destructuring

struct Point {
    x: i32,
    y: i32,
}

#[allow(dead_code)] // not every variant is built below
enum Color {
    Rgb(i32, i32, i32),
    Hsv(i32, i32, i32),
}

#[allow(dead_code)]
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(Color),
}

fn main() {
    // ---------- Structs ----------
    let p = Point { x: 0, y: 7 };

    let Point { x: a, y: b } = p; // field x into a new variable `a`
    println!("struct:    a = {a}, b = {b}");

    let Point { x, y } = p; // shorthand: variables named after the fields
    println!("struct:    x = {x}, y = {y}");
    // let Point { x } = p;
    //   // E0027: pattern does not mention field `y` (add `y`, `y: _` or `..`)

    // Literals inside a struct pattern test some fields and bind others:
    for p in [
        Point { x: 0, y: 0 },
        Point { x: 3, y: 0 },
        Point { x: 0, y: 7 },
        Point { x: 2, y: 5 },
    ] {
        match p {
            Point { x: 0, y: 0 } => println!("struct:    the origin"),
            Point { x, y: 0 } => println!("struct:    on the x axis at {x}"),
            Point { x: 0, y } => println!("struct:    on the y axis at {y}"),
            Point { x, y } => println!("struct:    on neither axis: ({x}, {y})"),
        }
    }

    // ---------- Enums ----------
    // The pattern has the same shape as the variant: nothing, {fields},
    // or (tuple).
    let messages = [
        Message::Quit,
        Message::Move { x: 3, y: -1 },
        Message::Write(String::from("hello")),
        Message::ChangeColor(Color::Rgb(0, 160, 255)),
        Message::ChangeColor(Color::Hsv(200, 100, 50)),
    ];
    for msg in &messages {
        // `msg` is a &Message. Matching a reference with non-reference
        // patterns BORROWS the fields (see the binding_modes example).
        match msg {
            Message::Quit => println!("enum:      quit"),
            Message::Move { x, y } => println!("enum:      move by ({x}, {y})"),
            Message::Write(text) => println!("enum:      text {text:?}"),
            // ---------- Nested: an enum inside an enum ----------
            Message::ChangeColor(Color::Rgb(r, g, b)) => {
                println!("nested:    red {r}, green {g}, blue {b}")
            }
            Message::ChangeColor(Color::Hsv(h, s, v)) => {
                println!("nested:    hue {h}, saturation {s}, value {v}")
            }
        }
    }

    // ---------- Structs and tuples together ----------
    let ((feet, inches), Point { x, y }) = ((3, 10), Point { x: 3, y: -10 });
    println!("mixed:     {feet}' {inches}\", point ({x}, {y})");

    // ---------- References ----------
    // `&Point { x, y }` matches a &Point and copies the i32 fields out, so
    // the closure works with plain numbers.
    let points = [Point { x: 1, y: 2 }, Point { x: 3, y: 4 }];
    let sum_of_squares: i32 = points.iter().map(|&Point { x, y }| x * x + y * y).sum();
    println!("reference: sum of squares = {sum_of_squares}");
}
