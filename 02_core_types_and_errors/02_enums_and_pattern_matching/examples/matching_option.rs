//! Matching `Option<T>`, and why matches must be exhaustive.
//! Run: cargo run -p enums_and_pattern_matching --example matching_option

// Clippy would suggest `x.map(|i| i + 1)` (see option_combinators). We keep
// the Book's explicit match here because the match IS the lesson.
#[allow(clippy::manual_map)]
fn plus_one(x: Option<i32>) -> Option<i32> {
    match x {
        None => None,
        Some(i) => Some(i + 1),
    }
}

#[allow(clippy::manual_map)]
fn plus_one_broken(x: Option<i32>) -> Option<i32> {
    match x {
        Some(i) => Some(i + 1),
        // Delete the next line to see:
        // E0004: non-exhaustive patterns: `None` not covered
        None => None,
    }
}

fn main() {
    let five = Some(5);
    let six = plus_one(five);
    let none = plus_one(None);
    println!("{five:?} -> {six:?}, None -> {none:?}");
    println!("{:?}", plus_one_broken(Some(41)));

    // This is the billion-dollar-mistake fix: the compiler will not let you
    // forget the "nothing there" case, because a match must cover them all.
}
