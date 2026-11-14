//! Book 19.3 (part 1): literals, named variables, `|` and ranges.
//! Run: cargo run -p patterns_and_matching --example literals_variables_ranges

fn main() {
    // ---------- Matching literals ----------
    let x = 1;
    match x {
        1 => println!("literal:   one"),
        2 => println!("literal:   two"),
        3 => println!("literal:   three"),
        _ => println!("literal:   anything"),
    }

    // ---------- Named variables (and the shadowing trap) ----------
    // A plain name in a pattern doesn't COMPARE with an outer variable: it
    // creates a NEW variable that matches anything.
    let x = Some(5);
    let y = 10;
    match x {
        Some(50) => println!("named:     got 50"),
        Some(y) => println!("named:     matched, y = {y}"), // a new y, = 5, shadows the outer one
        _ => println!("named:     default case, x = {x:?}"),
    }
    println!("named:     at the end: x = {x:?}, y = {y}"); // the outer y is still 10
    // To really compare with the outer y, use a match guard: `Some(n) if n == y`
    // (see the guards_and_bindings example).

    // ---------- Multiple patterns with | ----------
    let x = 1;
    match x {
        1 | 2 => println!("or:        one or two"),
        3 => println!("or:        three"),
        _ => println!("or:        anything"),
    }

    // ---------- Ranges with ..= ----------
    // Inclusive ranges work for numbers and chars. The compiler checks the
    // range isn't empty and counts it towards exhaustiveness.
    for x in [3, 7] {
        match x {
            1..=5 => println!("range:     {x} is one through five"),
            _ => println!("range:     {x} is something else"),
        }
    }
    for c in ['c', 'X', '7'] {
        let kind = match c {
            'a'..='j' => "early ASCII letter",
            'k'..='z' => "late ASCII letter",
            'A'..='Z' => "capital",
            _ => "something else",
        };
        println!("range:     {c:?} is {kind}");
    }

    // Exclusive (a..b) and half-open (a.., ..=b) ranges work too, and an
    // integer match can be exhaustive WITHOUT a `_`:
    for temp in [-4_i8, 0, 18, 31] {
        let feel = match temp {
            ..0 => "freezing",
            0..15 => "cold",
            15..=25 => "nice",
            26.. => "hot",
        };
        println!("range:     {temp} C is {feel}");
    }
}
