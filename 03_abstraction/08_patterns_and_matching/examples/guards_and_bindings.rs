//! Book 19.3 (part 4): extra conditionals with match guards, and `@`
//! bindings.
//! Run: cargo run -p patterns_and_matching --example guards_and_bindings

enum Message {
    Hello { id: i32 },
}

fn main() {
    // ---------- Match guards: `PATTERN if CONDITION` ----------
    // The pattern matches first, THEN the guard must be true too.
    for num in [Some(4), Some(5), None] {
        match num {
            Some(x) if x % 2 == 0 => println!("guard:     the number {x} is even"),
            Some(x) => println!("guard:     the number {x} is odd"),
            None => println!("guard:     no number"),
        }
    }

    // A guard can use OUTER variables, which fixes the shadowing trap from
    // the literals_variables_ranges example: `Some(y)` would create a new y.
    let x = Some(5);
    let y = 10;
    match x {
        Some(50) => println!("outer:     got 50"),
        Some(n) if n == y => println!("outer:     matched, n = {n}"),
        _ => println!("outer:     default case, x = {x:?}"),
    }
    println!("outer:     at the end: x = {x:?}, y = {y}");

    // The guard applies to the WHOLE `|` pattern:
    //   (4 | 5 | 6) if y    NOT    4 | 5 | (6 if y)
    let x = 4;
    let y = false;
    match x {
        // (The Book's example. Clippy would suggest the range `4..=6`.)
        #[allow(clippy::manual_range_patterns)]
        4 | 5 | 6 if y => println!("or+guard:  yes"),
        _ => println!("or+guard:  no (x is 4, but the guard is false for every alternative)"),
    }

    // The compiler doesn't look inside guards when it checks exhaustiveness,
    // so guarded arms need a fallback even when they "obviously" cover
    // everything:
    let n: u8 = 7;
    let parity = match n {
        n if n % 2 == 0 => "even",
        n if n % 2 == 1 => "odd",
        _ => unreachable!("every u8 is even or odd"), // without this: E0004
    };
    println!("guards:    {n} is {parity}");

    // ---------- @ bindings: test a value AND keep it ----------
    for msg in [
        Message::Hello { id: 5 },
        Message::Hello { id: 11 },
        Message::Hello { id: 42 },
    ] {
        match msg {
            // `id @ 3..=7`: the field must be in 3..=7, and is bound to `id`.
            Message::Hello { id: id @ 3..=7 } => println!("@:         found an id in range: {id}"),
            // A range alone tests, but leaves nothing to print:
            Message::Hello { id: 10..=12 } => println!("@:         found an id in another range"),
            Message::Hello { id } => println!("@:         found some other id: {id}"),
        }
    }

    // @ works with any sub-pattern, including | alternatives (in brackets):
    for c in ['a', 'q', '?'] {
        match c {
            vowel @ ('a' | 'e' | 'i' | 'o' | 'u') => println!("@:         {vowel:?} is a vowel"),
            letter @ 'a'..='z' => println!("@:         {letter:?} is a consonant"),
            other => println!("@:         {other:?} is not a lowercase letter"),
        }
    }
}
