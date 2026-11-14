//! What a procedural macro works with: tokens, a `syn` syntax tree, and
//! `quote!`. All three also run in an ordinary program, which is how you
//! explore them (and how this lesson unit-tests its macros).
//!
//! Run: cargo run -p macros_in_depth --example syn_and_quote

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields};

fn main() -> syn::Result<()> {
    // 1. A derive macro receives the struct as TOKENS. Here we make them
    //    from a string; the compiler hands a real macro a `TokenStream`.
    let tokens: TokenStream = r#"
        /// A point on screen.
        #[derive(Debug)]
        pub struct Point { x: i32, pub y: i32 }
    "#
    .parse()?;
    println!("1. tokens: a flat list of token trees (groups nest):");
    for tree in tokens.clone() {
        let kind = match &tree {
            TokenTree::Ident(_) => "Ident  ",
            TokenTree::Punct(_) => "Punct  ",
            TokenTree::Literal(_) => "Literal",
            TokenTree::Group(_) => "Group  ",
        };
        println!("   {kind} {tree}");
    }

    // 2. `syn` parses the tokens into a syntax tree. `DeriveInput` is what
    //    every derive macro starts from: attributes, visibility, name,
    //    generics, and the data (struct fields or enum variants).
    let input: DeriveInput = syn::parse2(tokens)?;
    println!("\n2. syn::DeriveInput:");
    println!("   name: {}", input.ident);
    println!("   attributes: {}", input.attrs.len()); // the doc comment and the derive
    let Data::Struct(data) = &input.data else {
        unreachable!("we wrote a struct")
    };
    let Fields::Named(fields) = &data.fields else {
        unreachable!("with named fields")
    };
    for field in &fields.named {
        let ty = &field.ty;
        println!(
            "   field `{}`: {} (public: {})",
            field.ident.as_ref().unwrap(),
            quote!(#ty),
            matches!(field.vis, syn::Visibility::Public(_))
        );
    }

    // 3. `quote!` writes code: `#name` pastes one value, `#(#names),*`
    //    repeats like `$(…),*` in `macro_rules!`. `format_ident!` makes new
    //    identifiers.
    let name = &input.ident;
    let field_names: Vec<_> = fields.named.iter().map(|f| f.ident.clone()).collect();
    let getters = field_names.iter().map(|field| {
        let getter = format_ident!("get_{}", field.as_ref().unwrap());
        quote! {
            pub fn #getter(&self) -> i32 { self.#field }
        }
    });
    let generated = quote! {
        impl #name {
            #(#getters)*
            pub fn field_names() -> &'static [&'static str] {
                &[#(stringify!(#field_names)),*]
            }
        }
    };
    println!("\n3. quote! output (what a derive hands back):\n   {generated}");

    // 4. Errors carry a SPAN: where in the user's code to point. Outside the
    //    compiler, spans only know line and column when proc-macro2's
    //    "span-locations" feature is on, so here we just show the message.
    //    Inside a macro, `to_compile_error()` turns this into
    //    `compile_error!("…")` with the span attached.
    let enum_input: DeriveInput = syn::parse_str("enum Mode { Fast, Slow }")?;
    let error = match &enum_input.data {
        Data::Enum(data) => syn::Error::new(data.enum_token.span(), "expected a struct"),
        _ => unreachable!(),
    };
    println!("\n4. an error, as a macro would return it:");
    println!("   {}", error.to_compile_error());

    // 5. Span::call_site() is "where the macro was called"; mixed_site() is
    //    `macro_rules!`-style hygiene for local variables.
    let hygienic = proc_macro2::Ident::new("temp", Span::mixed_site());
    println!("\n5. an identifier with mixed-site hygiene: {hygienic}");
    Ok(())
}
