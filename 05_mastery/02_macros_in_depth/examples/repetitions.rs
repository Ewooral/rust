//! Repetitions: `$( … ),*`, `+`, `?`, separators, nesting, and the errors
//! you get when the counts don't line up.
//!
//! Run: cargo run -p macros_in_depth --example repetitions

/// `+` = one or more, separated by commas; `$(,)?` = an optional trailing comma.
/// (`*` would be zero or more.)
macro_rules! my_vec {
    // Empty gets its own rule. Without it, `my_vec![]` expands to
    // `let mut v = Vec::new(); v`, and the compiler warns "variable does not
    // need to be mutable", pointing at the CALLER's `my_vec![]`.
    () => {
        Vec::new()
    };
    ($($item:expr),+ $(,)?) => {{
        let mut v = Vec::new();
        $( v.push($item); )* // one push per matched $item
        v
    }};
}

/// The separator can be any token, here `;`.
macro_rules! sum_lines {
    ($($n:expr);+) => { 0 $(+ $n)+ };
}

/// `?` = zero or one: an optional `as <type>`.
macro_rules! parse_or_zero {
    ($text:expr $(, as $t:ty)?) => {
        $text.trim().parse $(::<$t>)? ().unwrap_or_default()
    };
}

/// Two metavariables in ONE repetition repeat together, in step.
macro_rules! struct_with_getters {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Debug, Default)]
        struct $name { $($field: $ty),* }

        impl $name {
            $(
                fn $field(&self) -> &$ty {
                    &self.$field
                }
            )*
        }
    };
}

struct_with_getters!(Book {
    title: String,
    pages: u32
});

/// Nested repetitions: groups of members. Each `$(…)*` in the expansion
/// must be at the same depth as in the pattern.
macro_rules! teams {
    ($($team:ident: [$($member:literal),*]);* $(;)?) => {{
        let mut lines = Vec::new();
        $(
            let members: Vec<&str> = vec![$($member),*];
            lines.push(format!("{} ({}): {}", stringify!($team), members.len(), members.join(", ")));
        )*
        lines
    }};
}

/// Counting: replace each token with `1` and add them up. (`stringify!`
/// gives each repetition something to repeat with, and is thrown away.)
macro_rules! count {
    ($($x:tt)*) => { 0usize $(+ { let _ = stringify!($x); 1usize })* };
}

fn main() {
    println!("my_vec!     = {:?}", my_vec![1, 2, 3,]);
    println!("empty       = {:?}", my_vec![] as Vec<u8>);
    println!("sum_lines!  = {}", sum_lines!(1; 2; 3 * 4));
    println!("parse u8    = {}", parse_or_zero!(" 42 ", as u8));
    println!("parse bad   = {}", parse_or_zero!("x", as i32));

    let book = Book {
        title: "The Rust Book".into(),
        pages: 550,
    };
    println!("getters     = {} / {} pages", book.title(), book.pages());
    println!("default     = {:?}", Book::default());

    for line in teams!(red: ["Ada", "Alan"]; blue: ["Grace"]; empty: []) {
        println!("teams!      : {line}");
    }
    println!("count!      = {}", count!(a b (c d) [e]));

    // Mistakes with repetitions (none has an error code):
    //
    // 1. Using a repeated metavariable outside its `$(…)*`:
    //      macro_rules! d { ($($a:ident),*) => { $a } }
    //      error: variable `a` is still repeating at this depth
    //
    // 2. Two metavariables in one repetition, matched different numbers of times:
    //      macro_rules! zip { ($($a:ident),* ; $($b:ident),*) => { $( let $a = $b; )* } }
    //      zip!(p, q ; x, y, z);
    //      error: meta-variable `a` repeats 2 times, but `b` repeats 3 times
    //
    // 3. A pattern that could split the input more than one way:
    //      macro_rules! amb { ($($a:ident)* $b:ident) => {} }
    //      amb!(x y z);
    //      error: local ambiguity when calling macro `amb`: multiple parsing options:
    //             built-in NTs ident ('a') or ident ('b').
}
