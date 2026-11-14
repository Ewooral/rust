//! `palette! { … }`: the function-like macro, with its own little syntax.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Error, Ident, LitStr, Token, Visibility, braced};

/// The whole input: `pub mod theme { primary = "#336699", accent = "#f80" }`.
pub(crate) struct Palette {
    vis: Visibility,
    name: Ident,
    entries: Punctuated<Entry, Token![,]>,
}

/// One `name = "#rrggbb"` pair.
struct Entry {
    name: Ident,
    colour: LitStr,
}

// A function-like macro accepts ANY tokens, so it defines its own grammar by
// implementing `syn::parse::Parse`. Each `input.parse()?` either consumes the
// next piece or fails with an error at the token it didn't expect.
impl Parse for Palette {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis: Visibility = input.parse()?; // `pub`, `pub(crate)` or nothing
        input.parse::<Token![mod]>()?;
        let name: Ident = input.parse()?;
        let content;
        braced!(content in input); // the `{ … }`; `content` is what's inside
        let entries = Punctuated::parse_terminated(&content)?; // commas, trailing one allowed
        Ok(Palette { vis, name, entries })
    }
}

impl Parse for Entry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let colour: LitStr = input.parse()?;
        Ok(Entry { name, colour })
    }
}

/// `"#f80"` or `"#ff8800"` to `(r, g, b)`.
fn parse_hex(text: &str) -> Option<(u8, u8, u8)> {
    let digits = text.strip_prefix('#')?;
    if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let channel = |s: &str| u8::from_str_radix(s, 16).ok();
    match digits.len() {
        // "#f80" means "#ff8800": each digit is doubled.
        3 => {
            let d: Vec<String> = digits.chars().map(|c| format!("{c}{c}")).collect();
            Some((channel(&d[0])?, channel(&d[1])?, channel(&d[2])?))
        }
        6 => Some((
            channel(&digits[0..2])?,
            channel(&digits[2..4])?,
            channel(&digits[4..6])?,
        )),
        _ => None,
    }
}

pub(crate) fn expand(input: &Palette) -> syn::Result<TokenStream> {
    let mut errors: Vec<Error> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    let mut constants = Vec::new();
    let mut all = Vec::new();

    for entry in &input.entries {
        let name = entry.name.to_string();
        if seen.contains(&name) {
            errors.push(Error::new(
                entry.name.span(),
                format!("`{name}` is in this palette twice"),
            ));
        }
        seen.push(name.clone());

        // The check a `macro_rules!` macro can't do: look INSIDE the string.
        let Some((r, g, b)) = parse_hex(&entry.colour.value()) else {
            errors.push(Error::new(
                entry.colour.span(),
                "expected a colour like \"#ff8800\" or \"#f80\"",
            ));
            continue;
        };
        // The constant keeps the user's span, so "unused constant" warnings
        // and go-to-definition point at their name.
        let constant = format_ident!("{}", name.to_uppercase(), span = entry.name.span());
        constants.push(quote! {
            pub const #constant: ::macros_in_depth::Rgb = ::macros_in_depth::Rgb::new(#r, #g, #b);
        });
        all.push(quote!((#name, #constant)));
    }

    // Report every bad entry at once.
    if let Some(mut first) = errors.first().cloned() {
        for error in errors.into_iter().skip(1) {
            first.combine(error);
        }
        return Err(first);
    }

    let vis = &input.vis;
    let module = &input.name;
    Ok(quote! {
        #vis mod #module {
            #(#constants)*

            /// Every colour in this palette, by name.
            pub const ALL: &[(&str, ::macros_in_depth::Rgb)] = &[#(#all),*];
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expand_str(source: &str) -> syn::Result<String> {
        let palette: Palette = syn::parse_str(source)?;
        expand(&palette).map(|tokens| tokens.to_string())
    }

    #[test]
    fn reads_both_hex_forms() {
        assert_eq!(parse_hex("#ff8800"), Some((255, 136, 0)));
        assert_eq!(parse_hex("#f80"), Some((255, 136, 0)));
        assert_eq!(parse_hex("ff8800"), None);
        assert_eq!(parse_hex("#ff880"), None);
        assert_eq!(parse_hex("#gg8800"), None);
    }

    #[test]
    fn makes_a_module_of_constants() {
        let out =
            expand_str(r##"pub mod theme { primary = "#336699", accent = "#f80", }"##).unwrap();
        assert!(out.starts_with("pub mod theme"));
        assert!(out.contains("pub const PRIMARY : :: macros_in_depth :: Rgb = :: macros_in_depth :: Rgb :: new (51u8 , 102u8 , 153u8)"));
        assert!(out.contains("(\"accent\" , ACCENT)"));
    }

    #[test]
    fn bad_syntax_is_a_parse_error() {
        let error = expand_str(r##"mod theme { primary: "#336699" }"##).unwrap_err();
        assert_eq!(error.to_string(), "expected `=`");
        let error = expand_str(r##"theme { primary = "#336699" }"##).unwrap_err();
        assert_eq!(error.to_string(), "expected `mod`");
    }

    #[test]
    fn reports_every_bad_colour_and_duplicate() {
        let error = syn::parse_str::<Palette>(r##"mod t { a = "red", b = "#12", a = "#000" }"##)
            .and_then(|palette| expand(&palette))
            .unwrap_err();
        let messages: Vec<String> = error.into_iter().map(|e| e.to_string()).collect();
        assert_eq!(messages.len(), 3);
        assert!(messages[0].starts_with("expected a colour"));
        assert!(messages[2].contains("twice"));
    }
}
