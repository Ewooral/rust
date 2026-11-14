//! Debugging macros: print what a macro received, see what it expands to,
//! and fail with a clear message instead of a confusing one.
//!
//! Run: cargo run -p macros_in_depth --example debugging_macros
//!
//! The best tool is `cargo expand` (a separate tool, not part of Rust).
//! Installed with `cargo install cargo-expand`, it prints a crate or one
//! target after every macro has expanded:
//!
//!     cargo expand -p macros_in_depth --example debugging_macros
//!
//! It needs a nightly toolchain installed alongside stable (it uses the
//! compiler's unstable `-Zunpretty=expanded` option for you). This course
//! doesn't install it; the tricks below need nothing extra.

/// 1. `stringify!` turns tokens into a string literal, at compile time. Use
///    it to see exactly which tokens each metavariable captured.
macro_rules! show_captures {
    ($name:ident = $value:expr; $($rest:tt)*) => {
        println!("  name:  {}", stringify!($name));
        println!("  value: {}", stringify!($value));
        println!("  rest:  {}", stringify!($($rest)*));
    };
}

/// 2. A macro that also returns its expansion as text: a poor man's
///    `cargo expand` for one macro.
macro_rules! squares {
    ($($n:literal),*) => {
        (
            vec![$($n * $n),*],
            stringify!(vec![$($n * $n),*]),
        )
    };
}

/// 3. `compile_error!` in a last, catch-all rule turns "no rules expected
///    the token" into a message that says what to write instead.
macro_rules! percent {
    ($n:literal %) => {
        $n as f64 / 100.0
    };
    ($($other:tt)*) => {
        compile_error!(concat!(
            "percent! expects a number followed by %, like `percent!(15 %)`, not `",
            stringify!($($other)*),
            "`"
        ))
    };
}

/// Without a catch-all rule, a bad call gets the compiler's generic message.
macro_rules! two_args {
    ($a:expr, $b:expr) => {
        $a + $b
    };
}

fn main() {
    println!("show_captures!(total = 1 + 2 * 3; and then some):");
    show_captures!(total = 1 + 2 * 3; and then some);

    let (values, expansion) = squares!(1, 2, 3);
    println!("\nsquares!(1, 2, 3) expanded to `{expansion}` = {values:?}");

    println!("\npercent!(15 %) = {}", percent!(15 %));
    // percent!(fifteen);
    // ^ error: percent! expects a number followed by %, like `percent!(15 %)`, not `fifteen`

    println!("two_args!(1, 2) = {}", two_args!(1, 2));
    // two_args!(1);
    // ^ error: unexpected end of macro invocation   (no error code)
    //   note: while trying to match `,`
    // two_args!(1, 2, 3);
    // ^ error: no rules expected `,`   (no error code)

    // Nightly-only helpers you may see in older articles: `trace_macros!(true)`
    // prints every expansion step, and `log_syntax!` prints its input. On
    // stable they are errors (E0658: use of unstable library feature).
}
