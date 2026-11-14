//! Hygiene (which names a macro can see), `$crate`, and where a macro can
//! be used: textual scope, `#[macro_export]` and `pub(crate) use`.
//!
//! Run: cargo run -p macros_in_depth --example hygiene_and_crate_paths

// 1. HYGIENE: a local variable made inside a macro is a DIFFERENT variable
//    from a caller's variable with the same name.
macro_rules! double_and_report {
    ($e:expr) => {{
        let value = $e * 2; // the macro's own `value`
        format!("doubled: {value}")
    }};
}

// ...and the other way round: names written in the macro are looked up
// where the macro was DEFINED, not where it's used. So a macro defined out
// here can't see a caller's locals:
//     macro_rules! double_x { () => { x * 2 } }   // defined at the top level
//     fn main() { let x = 21; double_x!(); }
//     error[E0425]: cannot find value `x` in this scope
// Pass the name in (`$x:ident`), and it carries the CALLER's context:
macro_rules! double_it {
    ($x:ident) => {
        $x * 2
    };
}

// Hygiene covers local variables, labels and `$crate`. Items (functions,
// structs) are NOT hygienic: a fn defined by a macro is visible to the caller.
macro_rules! make_greeter {
    ($name:ident, $greeting:literal) => {
        fn $name(who: &str) -> String {
            format!("{}, {who}!", $greeting)
        }
    };
}
make_greeter!(hello, "Hello");

// 2. TEXTUAL SCOPE: a `macro_rules!` macro exists from its definition to the
//    end of the enclosing module (and, with `#[macro_use]` on a `mod`, after
//    it). Using it above its definition fails:
//     error: cannot find macro `early` in this scope   (no error code)
//
// 3. PATH-BASED SCOPE: `pub(crate) use` turns a macro into an item that other
//    modules import by path, like a function.
mod tools {
    pub mod numbers {
        pub fn triple(n: i32) -> i32 {
            n * 3
        }
    }

    // `$crate` means "the crate this macro was defined in", from wherever it
    // is used. Writing `numbers::triple($e)` here works in THIS module only;
    // from `main` it would fail with
    //     error[E0433]: cannot find module or crate `numbers` in this scope
    macro_rules! triple {
        ($e:expr) => {
            $crate::tools::numbers::triple($e)
        };
    }
    pub(crate) use triple; // now `crate::tools::triple!` is a path
}

// 4. `#[macro_export]` puts a macro at the CRATE ROOT, for other crates:
//    users write `my_crate::shout!` or `use my_crate::shout;`. Its helpers
//    must then be reached through `$crate::…`, as `macros_in_depth`'s
//    `settings_map!` does with `$crate::__private::to_value`.
#[macro_export]
macro_rules! shout {
    ($text:expr) => {
        $text.to_uppercase()
    };
}

fn main() {
    let value = 10; // the caller's `value`...
    println!("{}", double_and_report!(value + 1));
    println!("caller's value is still {value}"); // ...untouched

    let x = 21;
    println!("double_it!(x) = {}", double_it!(x));
    println!("{}", hello("Ada")); // a fn the macro defined

    use tools::triple;
    println!("triple!(5) = {}", triple!(5));
    println!("{}", crate::shout!("macros are code that writes code"));

    // The library's exported macros, from another crate (this example):
    let map = macros_in_depth::settings_map! { "port" => 8080 };
    println!("settings_map! from the library: {map:?}");
}
