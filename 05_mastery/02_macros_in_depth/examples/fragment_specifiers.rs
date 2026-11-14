//! `macro_rules!` fragment specifiers: what each `$name:kind` matches, and
//! what may follow it.
//!
//! Run: cargo run -p macros_in_depth --example fragment_specifiers

/// Tells you which rule matched. Rules are tried top to bottom, and the
/// first one whose pattern matches wins, so the order matters.
macro_rules! what_is_it {
    // Literal tokens in a pattern must match exactly: `type` here.
    (type $t:ty) => {
        format!("a type: {}", stringify!($t))
    };
    (lifetime $l:lifetime) => {
        format!("a lifetime: {}", stringify!($l))
    };
    (visibility $v:vis) => {
        format!("a visibility: {:?}", stringify!($v))
    };
    (path $p:path) => {
        format!("a path: {}", stringify!($p))
    };
    (pattern $p:pat) => {
        format!("a pattern: {}", stringify!($p))
    };
    (block $b:block) => {
        format!("a block that evaluates to {}", $b)
    };
    (item $i:item) => {{
        $i // the item (here a fn) is pasted into this block
        format!("an item, whose function returns {}", answer())
    }};
    (meta $m:meta) => {
        format!("an attribute's inside: {}", stringify!($m))
    };
    ($l:literal) => {
        format!("a literal: {}", stringify!($l))
    };
    ($i:ident) => {
        format!("an identifier: {}", stringify!($i))
    };
    ($e:expr) => {
        format!("an expression worth {}", $e)
    };
    // `tt` = one token tree: a single token, or a whole (...) [...] {...}
    // group. `$($t:tt)*` matches ANY input, so it goes last.
    ($($t:tt)*) => {
        format!("{} token trees: {}", 0 $(+ { stringify!($t); 1 })*, stringify!($($t)*))
    };
}

/// `expr` may only be followed by `=>`, `,` or `;`, so that a future Rust
/// with new operators can't change what an old macro matches:
///     macro_rules! bad { ($e:expr foo) => {} }
///     // error: `$e:expr` is followed by `foo`, which is not allowed for `expr` fragments
///     //        (no error code)  = note: allowed there are: `=>`, `,` or `;`
macro_rules! add_all {
    ($($e:expr),*) => { 0 $(+ $e)* };
}

/// In edition 2024, `expr` also matches `const { … }` blocks and `_`.
/// `expr_2021` keeps the older, narrower meaning.
macro_rules! old_style {
    ($e:expr_2021) => {
        $e
    };
}

fn main() {
    println!("{}", what_is_it!(type Vec<Option<u8>>));
    println!("{}", what_is_it!(lifetime 'a));
    println!("{}", what_is_it!(visibility pub(crate)));
    println!("{}", what_is_it!(path std::collections::HashMap));
    println!("{}", what_is_it!(pattern Some(1..=9) | None));
    println!("{}", what_is_it!(block { 6 * 7 }));
    println!("{}", what_is_it!(item fn answer() -> u8 { 42 }));
    println!("{}", what_is_it!(meta derive(Debug, Clone)));
    println!("{}", what_is_it!("hello"));
    println!("{}", what_is_it!(user_name));
    println!("{}", what_is_it!(2 + 3 * 4));
    println!("{}", what_is_it!(=> ?? [any] {thing}));

    println!("add_all!(1, 2 * 3, 4) = {}", add_all!(1, 2 * 3, 4));
    println!("old_style!(1 + 1) = {}", old_style!(1 + 1));
}
