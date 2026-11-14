//! Procedural macros for the advanced_features lesson (Book 20.5).
//!
//! A procedural macro is an ordinary Rust function that runs AT COMPILE
//! TIME. It receives code as a `TokenStream` and returns new code as a
//! `TokenStream`. There are three kinds, and this crate has one of each:
//!
//! | Kind           | Used as                          | Function here        |
//! |----------------|----------------------------------|----------------------|
//! | custom derive  | `#[derive(HelloMacro)]`          | `hello_macro_derive` |
//! | attribute-like | `#[trace]` on a function         | `trace`              |
//! | function-like  | `shout!("hello")`                | `shout`              |
//!
//! Real macros parse with the `syn` crate and build their output with
//! `quote!`. This crate has NO dependencies on purpose: it walks the tokens by
//! hand and builds code with `format!(..).parse()`, so nothing is hidden.
//!
//! Try them: `cargo run -p advanced_features --example procedural_macros`

use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

/// Custom derive: `#[derive(HelloMacro)]` writes this for you:
///
/// ```text
/// impl HelloMacro for Pancakes {
///     fn hello_macro() {
///         println!("Hello, Macro! My name is Pancakes!");
///     }
/// }
/// ```
///
/// A derive macro only ADDS code: the struct itself is left exactly as written.
#[proc_macro_derive(HelloMacro)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = input.into_iter().collect();

    // The type's name is the identifier right after `struct`, `enum` or `union`.
    let Some(position) = tokens.iter().position(|t| {
        matches!(t, TokenTree::Ident(i) if matches!(i.to_string().as_str(), "struct" | "enum" | "union"))
    }) else {
        return compile_error("#[derive(HelloMacro)] works on a struct, enum or union");
    };
    let Some(TokenTree::Ident(name)) = tokens.get(position + 1) else {
        return compile_error("expected a type name");
    };

    // Generics would need `impl<T> HelloMacro for Name<T>`, with every bound
    // copied across. That is exactly the bookkeeping `syn` does for you.
    if matches!(tokens.get(position + 2), Some(TokenTree::Punct(p)) if p.as_char() == '<') {
        return compile_error("this little HelloMacro derive does not support generic types");
    }

    format!(
        "impl HelloMacro for {name} {{
            fn hello_macro() {{
                println!(\"Hello, Macro! My name is {name}!\");
            }}
        }}"
    )
    .parse()
    .expect("the generated impl is valid Rust")
}

/// Attribute-like: `#[trace]` or `#[trace("label")]` on a function prints a
/// line when the function starts and another when it returns, however it
/// returns (normal end, early `return`, `?`, even a panic).
///
/// An attribute macro gets TWO token streams, the attribute's own arguments
/// and the item it's attached to, and REPLACES the item with its output.
/// Here the body `{ … }` becomes:
///
/// ```text
/// {
///     struct __TraceGuard;
///     impl Drop for __TraceGuard { fn drop(&mut self) { println!("<- leaving label"); } }
///     println!("-> entering label");
///     let _trace_guard = __TraceGuard;   // dropped when the function returns
///     { …the original body… }
/// }
/// ```
#[proc_macro_attribute]
pub fn trace(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut tokens: Vec<TokenTree> = item.into_iter().collect();

    let Some(fn_name) = tokens
        .iter()
        .position(|t| matches!(t, TokenTree::Ident(i) if i.to_string() == "fn"))
        .and_then(|i| tokens.get(i + 1))
        .map(|t| t.to_string())
    else {
        return compile_error("#[trace] only works on functions");
    };

    let mut args = attr.into_iter();
    let label = match (args.next(), args.next()) {
        (None, _) => fn_name,
        (Some(TokenTree::Literal(lit)), None) if lit.to_string().starts_with('"') => {
            lit.to_string().trim_matches('"').to_string()
        }
        _ => return compile_error("use #[trace] or #[trace(\"label\")]"),
    };

    // The function's body is its last token: one `{ … }` group.
    let body = match tokens.pop() {
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => g,
        _ => return compile_error("#[trace] needs a function with a body"),
    };

    let mut new_body: TokenStream = format!(
        "struct __TraceGuard;
        impl ::std::ops::Drop for __TraceGuard {{
            fn drop(&mut self) {{ ::std::println!(\"<- leaving {label}\"); }}
        }}
        ::std::println!(\"-> entering {label}\");
        let _trace_guard = __TraceGuard;"
    )
    .parse()
    .expect("the generated guard is valid Rust");
    // Keep the ORIGINAL body tokens (not a re-parsed copy) so their spans
    // survive: an error inside the body still points at the right line.
    new_body.extend([TokenTree::Group(body)]);

    tokens.push(TokenTree::Group(Group::new(Delimiter::Brace, new_body)));
    tokens.into_iter().collect()
}

/// Function-like: `shout!("hello, world")` becomes the literal `"HELLO, WORLD"`.
///
/// `macro_rules!` can move tokens around but can't look INSIDE a string
/// literal. A procedural macro is real Rust code, so it can do anything with
/// its input, at compile time. The program never calls `to_uppercase`.
#[proc_macro]
pub fn shout(input: TokenStream) -> TokenStream {
    let mut tokens = input.into_iter();
    match (tokens.next(), tokens.next()) {
        (Some(TokenTree::Literal(lit)), None) if lit.to_string().starts_with('"') => {
            let mut upper = String::new();
            let mut after_backslash = false;
            for c in lit.to_string().chars() {
                // Leave escape sequences such as \n and \u{..} as they are.
                if after_backslash || c == '\\' {
                    upper.push(c);
                    after_backslash = !after_backslash && c == '\\';
                } else {
                    upper.extend(c.to_uppercase());
                }
            }
            upper.parse().expect("an upper-cased string literal")
        }
        _ => compile_error("shout! takes one string literal, like shout!(\"hi\")"),
    }
}

/// Reports a problem the way the compiler does: as an error at the macro call.
fn compile_error(message: &str) -> TokenStream {
    format!("::core::compile_error!({message:?});")
        .parse()
        .expect("compile_error! call is valid Rust")
}
