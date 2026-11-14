//! Book 20.5 (declarative macros) and Beyond the Book: macros in depth:
//! repetition, hygiene, recursion.
//! Run: cargo run -p advanced_features --example declarative_macros
//!
//! A `macro_rules!` macro is a `match` on CODE: each rule has a pattern on
//! the left and the code to write on the right. Expansion happens at compile
//! time, before type checking.

use std::collections::HashMap;

// ---- The Book's simplified vec! ---------------------------------------------------
macro_rules! my_vec {
    ( $( $x:expr ),* ) => {
        {
            #[allow(unused_mut)] // my_vec![] never pushes, so `mut` is unused then
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            temp_vec
        }
    };
}

// ---- Fragment specifiers: what a $name may match -------------------------------
// expr (an expression), ident (a name), ty (a type), literal, pat, block, tt
// (any single token tree), item, path, lifetime, meta, vis
macro_rules! make_getter {
    ($field:ident: $ty:ty) => {
        fn $field(&self) -> &$ty {
            &self.$field
        }
    };
}

struct Book {
    title: String,
    pages: u32,
}

impl Book {
    make_getter!(title: String);
    make_getter!(pages: u32);
}

// ---- Repetition: *, + and ?, separators, nesting ---------------------------------
// $( … ),*   zero or more, comma-separated
// $( … );+   one or more, semicolon-separated
// $( … )?    zero or one
macro_rules! hashmap {
    ( $( $key:expr => $value:expr ),* $(,)? ) => {{
        let mut map = HashMap::new();
        $( map.insert($key, $value); )*
        map
    }};
}

// Nested repetition: the outer $( )* repeats per group, the inner per item.
macro_rules! groups {
    ( $( $group:ident: [ $( $member:expr ),* ] );* $(;)? ) => {{
        let mut out = Vec::new();
        $( out.push((stringify!($group), vec![ $( $member ),* ])); )*
        out
    }};
}

// ---- Recursion: a macro that calls itself --------------------------------------
// Rules are tried top to bottom. The first is the base case.
macro_rules! max {
    ($x:expr) => { $x };
    ($x:expr, $($rest:expr),+) => {{
        let first = $x;           // each argument is evaluated exactly once
        let rest = max!($($rest),+);
        if first > rest { first } else { rest }
    }};
}

// Counting at compile time: replace each item with 1 and add them up.
macro_rules! count {
    () => { 0usize };
    ($head:tt $($tail:tt)*) => { 1usize + count!($($tail)*) };
}

// Internal rules (a common convention): an `@name` prefix marks helper rules
// callers aren't meant to use.
macro_rules! shout_all {
    (@one $word:literal) => { concat!($word, "!") };
    ($($word:literal),*) => { [ $( shout_all!(@one $word) ),* ] };
}

// ---- Hygiene ------------------------------------------------------------------------
// Local variables a macro creates can't clash with yours: `temp` inside the
// macro and `temp` at the call site are different variables.
macro_rules! double_it {
    ($e:expr) => {{
        let temp = $e;
        temp * 2
    }};
}

// The flip side: a macro can't use a caller's variable unless it is PASSED
// in. Writing `x` inside the macro body would be E0425: cannot find value `x`.
macro_rules! add_to {
    ($var:ident, $amount:expr) => {
        $var += $amount
    };
}

// Expression-like macros wrap their body in `{{ … }}` so it's ONE expression;
// otherwise `let doubled = double_it!(…)` would see several statements.

fn main() {
    let v: Vec<u32> = my_vec![1, 2, 3];
    println!("my_vec![1, 2, 3] = {v:?}");
    let empty: Vec<u8> = my_vec![];
    println!("my_vec![] = {empty:?}");

    let book = Book {
        title: String::from("The Rust Programming Language"),
        pages: 560,
    };
    println!(
        "getters written by a macro: {} ({} pages)",
        book.title(),
        book.pages()
    );

    let ages = hashmap! { "Ama" => 31, "Kofi" => 27, };
    println!(
        "hashmap! made {} entries, Kofi is {}",
        ages.len(),
        ages["Kofi"]
    );

    let teams = groups! { red: [1, 2]; blue: [3, 4, 5] };
    println!("nested repetition: {teams:?}");

    println!("max!(3, 9, 4, 1) = {}", max!(3, 9, 4, 1));
    let mut calls = 0;
    let mut next = || {
        calls += 1;
        calls * 10
    };
    let biggest = max!(next(), next(), next());
    println!("max! of three calls = {biggest}, and next() ran {calls} times");

    const N: usize = count!(a b c d e);
    println!("count!(a b c d e) = {N}, known at compile time");
    println!("{:?}", shout_all!("hey", "you"));

    let temp = 10;
    let doubled = double_it!(temp + 1);
    println!("double_it!(temp + 1) = {doubled}, and our temp is still {temp}");

    let mut score = 1;
    add_to!(score, 41);
    println!("add_to!(score, 41) -> {score}");

    // max!() // error: unexpected end of macro invocation (no rule matches nothing)
    // To see what a macro expands to: `cargo install cargo-expand`, then
    // `cargo expand -p advanced_features --example declarative_macros`.
}
