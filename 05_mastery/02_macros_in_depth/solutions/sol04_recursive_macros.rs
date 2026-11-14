//! Solution to exercise 4: write three `macro_rules!` macros (pass 3 tests).
//!
//! The rules (patterns) are written for you. Each expansion is a
//! placeholder that has the right TYPE (so the tests compile) but panics
//! with `todo!()`. Replace every expansion:
//!
//!   count!(a b (c d) [e])     the number of token trees: 4. `count!()` is 0.
//!                             Recurse: 1 + count!(the rest). Use `0usize`
//!                             and `1usize` so the result is a `usize`.
//!   max!(a, b, c)             the largest argument. Recurse on the rest, and
//!                             bind each argument with `let` so it's
//!                             evaluated ONCE (a test counts the calls).
//!   hash_map!{ k => v, … }    a `std::collections::HashMap`; `hash_map!{}` is
//!                             empty, and a trailing comma is allowed. Write
//!                             paths in full (`::std::collections::HashMap`)
//!                             so callers don't need a `use`.
//!
//! Expansions that are expressions with statements need a block: `{{ … }}`.
//!
//! Test: cargo test -p macros_in_depth --features solutions --bin sol04_recursive_macros

macro_rules! count {
    () => {
        0usize
    };
    ($first:tt $($rest:tt)*) => {
        1usize + count!($($rest)*)
    };
}

macro_rules! max {
    ($x:expr) => {
        $x
    };
    ($x:expr, $($rest:expr),+) => {{
        let first = $x;
        let rest = max!($($rest),+);
        if first > rest { first } else { rest }
    }};
}

macro_rules! hash_map {
    () => {
        ::std::collections::HashMap::new()
    };
    ($($key:expr => $value:expr),+ $(,)?) => {{
        let mut map = ::std::collections::HashMap::new();
        $( map.insert($key, $value); )+
        map
    }};
}

fn main() {
    println!("count!(a b (c d) [e]) = {}", count!(a b (c d) [e]));
    println!("max!(3, 9, 4) = {}", max!(3, 9, 4));
    let ages: std::collections::HashMap<&str, u32> = hash_map! { "Ada" => 36, "Alan" => 41 };
    println!("{} ages", ages.len());
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::collections::HashMap;

    #[test]
    fn count_counts_token_trees() {
        assert_eq!(count!(), 0);
        assert_eq!(count!(x), 1);
        assert_eq!(count!(a b (c d) [e]), 4, "a group is one token tree");
        assert_eq!(count!(+ - * /), 4);
    }

    #[test]
    fn max_evaluates_each_argument_once() {
        assert_eq!(max!(7), 7);
        assert_eq!(max!(3, 9, 4), 9);
        assert_eq!(max!(-1.5, -0.5), -0.5);
        let calls = Cell::new(0);
        let next = |n: i32| {
            calls.set(calls.get() + 1);
            n
        };
        assert_eq!(max!(next(1), next(8), next(2), next(5)), 8);
        assert_eq!(calls.get(), 4, "each argument evaluated exactly once");
    }

    #[test]
    fn hash_map_builds_a_map() {
        let empty: HashMap<String, i32> = hash_map! {};
        assert!(empty.is_empty());
        let ages: HashMap<&str, i32> = hash_map! {
            "Ada" => 36,
            "Alan" => 41, // trailing comma
        };
        assert_eq!(ages.len(), 2);
        assert_eq!(ages["Alan"], 41);
    }
}
