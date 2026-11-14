//! Solution to exercise 5: a derive macro's logic, with syn and quote (pass 4 tests).
//!
//! A `#[derive(Describe)]` macro would turn
//!
//!     struct Point { x: i32, y: i32 }
//!
//! into
//!
//!     impl Describe for Point {
//!         fn describe(&self) -> String {
//!             format!("Point { x: {:?}, y: {:?} }", self.x, self.y)
//!         }
//!     }
//!
//! (with the braces of the format string doubled, `{{` and `}}`, as
//! `format!` needs). Real proc macros keep this logic in a plain function
//! that works on `proc_macro2` tokens, so it can be unit-tested like this.
//! Implement `expand_describe`:
//!
//!   - Named fields: build the format string at macro time, e.g. with
//!     `format!("{name} {{{{ {} }}}}", parts.join(", "))`, where each part is
//!     `"x: {:?}"`. A `String` inside `quote!` becomes a string literal.
//!     Then `#(self.#fields),*` repeats `self.x, self.y`.
//!   - A unit struct (`struct Marker;`, `Fields::Unit`): `format!("Marker")`.
//!   - Generics: use `input.generics.split_for_impl()` and write
//!     `impl #impl_generics Describe for #name #ty_generics #where_clause`.
//!   - A tuple struct: `Err` spanned on the fields, message
//!     "Describe needs named fields".
//!   - An enum or union: `Err` with the message
//!     "Describe only works on structs" (span: the `enum` token).
//!
//! Test: cargo test -p macros_in_depth --features solutions --bin sol05_derive_logic

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

fn expand_describe(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let body = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => {
                let fields: Vec<_> = named.named.iter().map(|f| &f.ident).collect();
                let parts: Vec<String> = fields
                    .iter()
                    .map(|f| format!("{}: {{:?}}", f.as_ref().expect("named")))
                    .collect();
                // `{{{{` in a format string is a literal `{{`.
                let template = format!("{name} {{{{ {} }}}}", parts.join(", "));
                quote!(format!(#template, #(self.#fields),*))
            }
            Fields::Unit => {
                let template = name.to_string();
                quote!(format!(#template))
            }
            Fields::Unnamed(fields) => {
                return Err(syn::Error::new_spanned(
                    fields,
                    "Describe needs named fields",
                ));
            }
        },
        Data::Enum(data) => {
            return Err(syn::Error::new(
                data.enum_token.span,
                "Describe only works on structs",
            ));
        }
        Data::Union(data) => {
            return Err(syn::Error::new(
                data.union_token.span,
                "Describe only works on structs",
            ));
        }
    };
    // `<T: Clone>` for the impl, `<T>` for the type, and the where clause.
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics Describe for #name #ty_generics #where_clause {
            fn describe(&self) -> String {
                #body
            }
        }
    })
}

fn main() {
    let input: DeriveInput = syn::parse_quote! {
        struct Point { x: i32, y: i32 }
    };
    match expand_describe(&input) {
        Ok(tokens) => println!("{tokens}"),
        Err(error) => println!("error: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    /// Compares token streams by their text, which ignores spacing.
    fn same(actual: TokenStream, expected: TokenStream) {
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn named_fields() {
        let out = expand_describe(&parse_quote! {
            struct Point { x: i32, y: i32 }
        });
        same(
            out.unwrap(),
            quote! {
                impl Describe for Point {
                    fn describe(&self) -> String {
                        format!("Point {{ x: {:?}, y: {:?} }}", self.x, self.y)
                    }
                }
            },
        );
    }

    #[test]
    fn unit_struct() {
        let out = expand_describe(&parse_quote!(
            struct Marker;
        ));
        same(
            out.unwrap(),
            quote! {
                impl Describe for Marker {
                    fn describe(&self) -> String {
                        format!("Marker")
                    }
                }
            },
        );
    }

    #[test]
    fn generics_are_carried_over() {
        let out = expand_describe(&parse_quote! {
            struct Wrapper<T: Clone> where T: Default { value: T }
        });
        same(
            out.unwrap(),
            quote! {
                impl<T: Clone> Describe for Wrapper<T> where T: Default {
                    fn describe(&self) -> String {
                        format!("Wrapper {{ value: {:?} }}", self.value)
                    }
                }
            },
        );
    }

    #[test]
    fn tuple_structs_and_enums_are_errors() {
        let tuple = expand_describe(&parse_quote!(
            struct Pair(u8, u8);
        ));
        assert_eq!(
            tuple.unwrap_err().to_string(),
            "Describe needs named fields"
        );
        let enumeration = expand_describe(&parse_quote!(
            enum Mode {
                Fast,
            }
        ));
        assert_eq!(
            enumeration.unwrap_err().to_string(),
            "Describe only works on structs"
        );
    }
}
