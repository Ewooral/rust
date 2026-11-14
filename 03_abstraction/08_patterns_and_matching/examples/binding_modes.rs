//! Beyond the Book: binding modes. Does a pattern MOVE, COPY or BORROW
//! what it binds? (`ref`, `ref mut`, `&`, and "match ergonomics")
//! Run: cargo run -p patterns_and_matching --example binding_modes

#[derive(Debug)]
enum Message {
    Write(String),
    Move { x: i32, y: i32 },
}

fn main() {
    // ---------- 1. By value: bindings MOVE (or copy) ----------
    let msg = Message::Write(String::from("hi"));
    if let Message::Write(text) = msg {
        println!("move:      took the String out: {text}");
    }
    // println!("{msg:?}");
    //   // E0382: borrow of partially moved value: `msg`

    // ---------- 2. Match a REFERENCE: bindings BORROW automatically ----------
    // Matching `&Message` against the pattern `Message::Write(text)` makes
    // `text` a `&String`. This is called "match ergonomics" (the default
    // binding mode switches to `ref`).
    let msg = Message::Write(String::from("hello"));
    if let Message::Write(text) = &msg {
        let _: &String = text; // proof of the type
        println!("borrow:    looked at {text:?}");
    }
    println!("borrow:    msg is still whole: {msg:?}");

    // ---------- 3. `ref`: borrow while matching a VALUE ----------
    // The older way to say the same thing. You'll see it in older code.
    if let Message::Write(ref text) = msg {
        println!("ref:       also borrowed: {text:?}");
    }

    // ---------- 4. Change things in place: &mut or `ref mut` ----------
    let mut position = Message::Move { x: 1, y: 2 };
    if let Message::Move { x, y } = &mut position {
        *x += 10; // x and y are &mut i32
        *y += 20;
    }
    let mut counter = Some(0);
    if let Some(ref mut n) = counter {
        *n += 1;
    }
    println!("&mut:      {position:?}, {counter:?}");

    // ---------- 5. `&` in a pattern: dereference and copy out ----------
    let pairs = vec![(1, 'a'), (2, 'b')];
    for &(number, letter) in &pairs {
        // number: i32, letter: char (copies), not references
        print!("({number}, {letter}) ");
    }
    println!(" <- the `&` pattern copied them out");

    // ---------- 6. The 2024 edition rule ----------
    // Once a pattern is borrowing IMPLICITLY (case 2), you may not also write
    // `ref`, `mut` or `&` inside it. These are errors (they have no code):
    //
    // if let Some(ref s) = &Some(String::new()) {}
    //   // error: cannot explicitly borrow within an implicitly-borrowing pattern
    // let [mut c] = &[5];
    //   // error: cannot mutably bind by value within an implicitly-borrowing pattern
    // let (x, &y) = &(1, &2);
    //   // error: cannot explicitly dereference within an implicitly-borrowing pattern
    //
    // The fix the compiler suggests: drop the modifier, or write the outer `&`
    // yourself so nothing is implicit: `let &[mut c] = &[5];`
    let &[mut c] = &[5];
    c += 1;
    println!("2024:      c = {c}");
}
