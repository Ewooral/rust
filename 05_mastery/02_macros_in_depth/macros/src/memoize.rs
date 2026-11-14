//! `#[memoize]`: the attribute macro.

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{Error, FnArg, Ident, ItemFn, LitInt, Pat, ReturnType, Type, parse_quote};

/// Wraps the function's body in a per-thread cache.
///
/// An attribute macro gets TWO token streams: the attribute's arguments
/// (`capacity = 100`) and the item it's attached to. Its output REPLACES the
/// item.
pub(crate) fn expand(args: TokenStream, mut function: ItemFn) -> syn::Result<TokenStream> {
    // 1. The arguments: nothing, or `capacity = <number>`.
    let mut capacity: Option<usize> = None;
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("capacity") {
            let number: LitInt = meta.value()?.parse()?;
            let n: usize = number.base10_parse()?;
            if n == 0 {
                return Err(Error::new_spanned(number, "capacity must be at least 1"));
            }
            capacity = Some(n);
            Ok(())
        } else {
            Err(meta.error("unknown memoize option: the only one is `capacity = <number>`"))
        }
    });
    parser.parse2(args)?;

    // 2. Check the function, pointing each error at the part that's wrong.
    let sig = &function.sig;
    if let Some(token) = &sig.asyncness {
        return Err(Error::new(
            token.span,
            "#[memoize] can't cache an `async fn`: its result is a future",
        ));
    }
    if !sig.generics.params.is_empty() {
        return Err(Error::new_spanned(
            &sig.generics,
            "#[memoize] can't cache a generic function: there would be one cache per type",
        ));
    }
    let output_type: Type = match &sig.output {
        ReturnType::Type(_, ty) => (**ty).clone(),
        ReturnType::Default => {
            return Err(Error::new_spanned(
                &sig.ident,
                "#[memoize] needs a function that returns a value: there is nothing to remember",
            ));
        }
    };

    let mut names: Vec<Ident> = Vec::new();
    let mut types: Vec<Type> = Vec::new();
    for input in &sig.inputs {
        match input {
            FnArg::Receiver(receiver) => {
                return Err(Error::new_spanned(
                    receiver,
                    "#[memoize] works on free functions, not methods: the cache can't key on `self`",
                ));
            }
            FnArg::Typed(typed) => {
                let Pat::Ident(pat) = &*typed.pat else {
                    return Err(Error::new_spanned(
                        &typed.pat,
                        "#[memoize] needs a plain name for each argument, like `n: u64`",
                    ));
                };
                match &*typed.ty {
                    Type::Reference(reference) => {
                        return Err(Error::new_spanned(
                            reference,
                            "#[memoize] keeps arguments in its cache, so they must be owned: use `String` instead of `&str`, `Vec<T>` instead of `&[T]`",
                        ));
                    }
                    Type::ImplTrait(impl_trait) => {
                        return Err(Error::new_spanned(
                            impl_trait,
                            "#[memoize] can't store `impl Trait` arguments: name a concrete type",
                        ));
                    }
                    _ => {}
                }
                names.push(pat.ident.clone());
                types.push((*typed.ty).clone());
            }
        }
    }

    // 3. Build the new body. `Span::mixed_site()` gives our local names the
    //    same hygiene as a `macro_rules!` local: the user's body can't see
    //    them, and can't clash with them. (Items such as `CACHE` are looked up
    //    at the call site either way, hence the unusual name.)
    let key = Ident::new("key", Span::mixed_site());
    let result = Ident::new("result", Span::mixed_site());
    let cache = Ident::new("cache", Span::mixed_site());
    let hit = Ident::new("hit", Span::mixed_site());

    // One check per argument, spanned to the argument's TYPE, so "the trait
    // `Hash` is not implemented for `Point`" points at `Point` in the
    // signature instead of at `#[memoize]`.
    let key_checks = types.iter().map(|ty| {
        quote_spanned! {ty.span()=> __memoize_assert_key::<#ty>(); }
    });
    let output_check = quote_spanned! {output_type.span()=>
        __memoize_assert_value::<#output_type>();
    };
    let make_room = match capacity {
        Some(limit) => quote! {
            if #cache.len() >= #limit {
                #cache.clear(); // simple: forget everything when full
            }
        },
        None => quote!(),
    };
    let original_body = &function.block;

    let new_body: syn::Block = parse_quote! {{
        fn __memoize_assert_key<T: ::core::clone::Clone + ::core::hash::Hash + ::core::cmp::Eq>() {}
        fn __memoize_assert_value<T: ::core::clone::Clone>() {}
        #(#key_checks)*
        #output_check

        ::std::thread_local! {
            static __MEMOIZE_CACHE: ::std::cell::RefCell<
                ::std::collections::HashMap<(#(#types,)*), #output_type>
            > = ::std::cell::RefCell::new(::std::collections::HashMap::new());
        }

        let #key = (#(::core::clone::Clone::clone(&#names),)*);
        let #hit = __MEMOIZE_CACHE.with(|#cache| #cache.borrow().get(&#key).cloned());
        if let ::core::option::Option::Some(#hit) = #hit {
            return #hit;
        }
        // The original body runs in a closure, so its `return`s and `?`s
        // still end "the function", and the cache is filled afterwards.
        // The RefCell isn't borrowed while it runs, so recursion works.
        let #result: #output_type = (move || -> #output_type #original_body)();
        __MEMOIZE_CACHE.with(|#cache| {
            let mut #cache = #cache.borrow_mut();
            #make_room
            #cache.insert(#key, ::core::clone::Clone::clone(&#result));
        });
        #result
    }};
    *function.block = new_body;

    // Everything else (attributes, `pub`, the signature) is kept as written.
    Ok(quote!(#function))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(args: TokenStream, function: ItemFn) -> syn::Result<String> {
        expand(args, function).map(|tokens| tokens.to_string())
    }

    #[test]
    fn wraps_the_body_and_keeps_the_signature() {
        let out = run(
            quote!(),
            parse_quote! {
                pub fn square(n: u64) -> u64 { n * n }
            },
        )
        .unwrap();
        assert!(out.starts_with("pub fn square (n : u64) -> u64"));
        assert!(out.contains("thread_local !"));
        assert!(out.contains("HashMap < (u64 ,) , u64 >"));
        assert!(
            out.contains("{ n * n }"),
            "the original body is still there"
        );
    }

    #[test]
    fn capacity_adds_a_size_check() {
        let out = run(
            quote!(capacity = 2),
            parse_quote!(
                fn f(n: u8) -> u8 {
                    n
                }
            ),
        )
        .unwrap();
        assert!(out.contains(". len () >= 2usize"));
    }

    #[test]
    fn explains_what_it_cannot_cache() {
        let cases: Vec<(TokenStream, ItemFn, &str)> = vec![
            (
                quote!(),
                parse_quote!(
                    async fn f(n: u8) -> u8 {
                        n
                    }
                ),
                "async fn",
            ),
            (
                quote!(),
                parse_quote!(
                    fn f<T>(n: T) -> u8 {
                        0
                    }
                ),
                "generic",
            ),
            (
                quote!(),
                parse_quote!(
                    fn f(n: u8) {}
                ),
                "returns a value",
            ),
            (
                quote!(),
                parse_quote!(
                    fn f(&self) -> u8 {
                        0
                    }
                ),
                "not methods",
            ),
            (
                quote!(),
                parse_quote!(
                    fn f(name: &str) -> u8 {
                        0
                    }
                ),
                "must be owned",
            ),
            (
                quote!(),
                parse_quote!(
                    fn f((a, b): (u8, u8)) -> u8 {
                        a
                    }
                ),
                "plain name",
            ),
            (
                quote!(size = 3),
                parse_quote!(
                    fn f(n: u8) -> u8 {
                        n
                    }
                ),
                "unknown memoize option",
            ),
            (
                quote!(capacity = 0),
                parse_quote!(
                    fn f(n: u8) -> u8 {
                        n
                    }
                ),
                "at least 1",
            ),
        ];
        for (args, function, expected) in cases {
            let error = run(args, function).expect_err(expected).to_string();
            assert!(
                error.contains(expected),
                "{error:?} should mention {expected:?}"
            );
        }
    }
}
