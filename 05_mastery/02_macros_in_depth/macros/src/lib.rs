//! Procedural macros for the `macros_in_depth` lesson, built with `syn` and
//! `quote`. One of each kind:
//!
//! | Kind           | Used as                             | Module        |
//! |----------------|-------------------------------------|---------------|
//! | derive         | `#[derive(Settings)]`               | [`settings`]  |
//! | attribute      | `#[memoize]` on a function          | [`memoize`]   |
//! | function-like  | `palette! { mod theme { … } }`      | [`palette`]   |
//!
//! Every macro has the same shape:
//!
//! 1. **parse** the `proc_macro::TokenStream` into a syntax tree with `syn`;
//! 2. **expand**: a normal function from the tree to a `proc_macro2::TokenStream`,
//!    returning `syn::Result`, so problems become errors with spans;
//! 3. **report**: `Ok` tokens go back to the compiler; an `Err` is turned into
//!    `compile_error!` calls by `syn::Error::to_compile_error`.
//!
//! Step 2 uses only `proc_macro2`, which also works outside the compiler, so
//! the unit tests in each module call it directly.
//!
//! The generated code names the support library by its absolute path,
//! `::macros_in_depth::…`: procedural macros have no `$crate`.

use proc_macro::TokenStream;
use syn::{DeriveInput, ItemFn, parse_macro_input};

mod memoize;
mod palette;
mod settings;

/// `#[derive(Settings)]`: load a struct from `key = value` text.
///
/// Implements `macros_in_depth::Settings`. Each field is read with `FromStr`;
/// `Option<T>` fields may be missing; `///` doc comments become the help
/// text. The helper attribute `#[setting(…)]` takes:
///
/// - `default = "8080"`: used when the key is missing, written as it would
///   appear in the settings file;
/// - `rename = "listen-port"`: the key to read instead of the field's name.
#[proc_macro_derive(Settings, attributes(setting))]
pub fn derive_settings(input: TokenStream) -> TokenStream {
    // `parse_macro_input!` returns early with a compile error if the input
    // isn't a struct, enum or union at all.
    let input = parse_macro_input!(input as DeriveInput);
    match settings::expand(&input) {
        Ok(tokens) => tokens.into(),
        // The long form of `unwrap_or_else(syn::Error::into_compile_error)`.
        Err(error) => error.to_compile_error().into(),
    }
}

/// `#[memoize]` or `#[memoize(capacity = 1000)]`: remember a function's
/// results, per thread, keyed by its arguments.
///
/// The function must take owned arguments that are `Clone + Hash + Eq`,
/// return a `Clone` value, and not be `async`, generic or a method.
#[proc_macro_attribute]
pub fn memoize(args: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    memoize::expand(args.into(), function)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// `palette! { pub mod theme { primary = "#336699", accent = "#f80" } }`:
/// colour constants checked at compile time.
///
/// Makes a module with one `macros_in_depth::Rgb` constant per entry (the
/// name in upper case) and `ALL`, a list of `(name, colour)` pairs.
#[proc_macro]
pub fn palette(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as palette::Palette);
    palette::expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
