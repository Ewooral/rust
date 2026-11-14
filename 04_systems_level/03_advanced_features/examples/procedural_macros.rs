//! Book 20.5: procedural macros, all three kinds.
//! Run: cargo run -p advanced_features --example procedural_macros
//!
//! The macros themselves are in ./macros/src/lib.rs, a separate crate of
//! type `proc-macro` (a procedural macro can't be used in the crate that
//! defines it). They are ordinary Rust functions that run INSIDE THE
//! COMPILER: tokens in, tokens out.
//!
//!   custom derive    #[derive(HelloMacro)]  adds an impl next to your type
//!   attribute-like   #[trace]               rewrites the item it's on
//!   function-like    shout!("…")            replaces the call with new code

use advanced_features::{HelloMacro, shout, trace};

// ---- Custom derive: the Book's HelloMacro --------------------------------------
// `use advanced_features::HelloMacro` imported BOTH the trait and the derive
// (they share a name, like serde's Serialize).
#[derive(HelloMacro)]
struct Pancakes;

#[derive(HelloMacro)]
#[allow(dead_code)]
enum Waffles {
    Plain,
    Chocolate,
}

// The derive refuses things it can't handle, with a real compile error:
// #[derive(HelloMacro)]
// struct Stack<T>(Vec<T>); // error: this little HelloMacro derive does not support generic types

// ---- Attribute-like: #[trace] ----------------------------------------------------
#[trace]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[trace("checking the order")]
fn check(order: &[u32]) -> Result<u32, String> {
    let total: u32 = order.iter().sum();
    if total == 0 {
        return Err(String::from("empty order")); // early return: still traced
    }
    Ok(total)
}

fn main() {
    println!("== custom derive ==");
    Pancakes::hello_macro();
    Waffles::hello_macro();

    println!("\n== attribute-like ==");
    let sum = add(2, 3);
    println!("add returned {sum}");
    println!("{:?}", check(&[3, 4]));
    println!("{:?}", check(&[]));

    println!("\n== function-like ==");
    // shout! runs at COMPILE time: the binary contains "HELLO, MACROS!".
    const LOUD: &str = shout!("hello, macros!");
    println!("{LOUD}");
    // Escapes stay escapes: \t is still a tab, \n still a new line (not \T, \N).
    println!("{}", shout!("one\ttwo\nthree"));
    // shout!(42); // error: shout! takes one string literal, like shout!("hi")

    // Real procedural macros use the `syn` crate to parse tokens into a
    // syntax tree and `quote!` to write code back. Compare the Book's version:
    //     let ast: syn::DeriveInput = syn::parse(input).unwrap();
    //     let name = &ast.ident;
    //     quote! { impl HelloMacro for #name { fn hello_macro() { … } } }
}
