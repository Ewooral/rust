//! `#[derive(Settings)]`: the derive macro.

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Error, Expr, Field, Fields, Ident, Lit, LitStr, Meta, Type};

/// What the derive learns about one field.
struct FieldPlan<'a> {
    field: &'a Field,
    key: String,
    default: Option<LitStr>,
    doc: String,
    optional: bool,
}

/// Turns the struct into an `impl ::macros_in_depth::Settings for …`.
pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;

    // 1. Check the SHAPE first, with an error pointing at the exact problem.
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => &named.named,
            other => {
                return Err(Error::new_spanned(
                    other,
                    "#[derive(Settings)] needs named fields, like `struct Config { port: u16 }`",
                ));
            }
        },
        Data::Enum(data) => {
            return Err(Error::new(
                data.enum_token.span,
                "#[derive(Settings)] only works on structs: an enum has no fields to fill in",
            ));
        }
        Data::Union(data) => {
            return Err(Error::new(
                data.union_token.span,
                "#[derive(Settings)] only works on structs",
            ));
        }
    };
    if !input.generics.params.is_empty() {
        return Err(Error::new_spanned(
            &input.generics,
            "#[derive(Settings)] doesn't support generic parameters",
        ));
    }

    // 2. Read every field. Collect ALL the problems, so the user sees them
    //    in one build instead of one per build: `Error::combine`.
    let mut plans = Vec::new();
    let mut errors: Option<Error> = None;
    for field in fields {
        match plan_field(field) {
            Ok(plan) => {
                if let Some(earlier) = plans.iter().find(|p: &&FieldPlan| p.key == plan.key) {
                    let message = format!(
                        "two fields read the key `{}` (the other one is `{}`)",
                        plan.key,
                        earlier.field.ident.as_ref().expect("named field")
                    );
                    push_error(&mut errors, Error::new_spanned(field, message));
                }
                plans.push(plan);
            }
            Err(error) => push_error(&mut errors, error),
        }
    }
    if let Some(errors) = errors {
        return Err(errors);
    }

    // 3. Generate the code. Every path is absolute (`::core::…`,
    //    `::macros_in_depth::…`), so the output works wherever it lands.
    let readers = plans.iter().map(|plan| {
        let ident = &plan.field.ident;
        let ty = &plan.field.ty;
        let key = &plan.key;
        let default = match &plan.default {
            Some(text) => quote!(::core::option::Option::Some(#text)),
            None => quote!(::core::option::Option::None),
        };
        // These tokens get the field TYPE's span. If the type doesn't
        // implement `FromStr`, the compiler blames the function being called,
        // so that name in particular must carry the span: then the error
        // points at the type in the user's struct, not at `#[derive(Settings)]`.
        let name = if plan.optional {
            "optional"
        } else {
            "required"
        };
        let function = Ident::new(name, ty.span());
        let read = quote_spanned! {ty.span()=>
            ::macros_in_depth::__private::#function(map, #key, #default)
        };
        quote!(#ident: #read?)
    });

    let infos = plans.iter().map(|plan| {
        let key = &plan.key;
        let ty = &plan.field.ty;
        let type_name = quote!(#ty).to_string().replace(' ', "");
        let doc = &plan.doc;
        let required = !plan.optional && plan.default.is_none();
        let default = match &plan.default {
            Some(text) => quote!(::core::option::Option::Some(#text)),
            None => quote!(::core::option::Option::None),
        };
        quote! {
            ::macros_in_depth::FieldInfo {
                key: #key,
                type_name: #type_name,
                default: #default,
                doc: #doc,
                required: #required,
            }
        }
    });

    Ok(quote! {
        impl ::macros_in_depth::Settings for #name {
            fn from_map(
                map: &::std::collections::BTreeMap<::std::string::String, ::std::string::String>,
            ) -> ::core::result::Result<Self, ::macros_in_depth::SettingsError> {
                ::core::result::Result::Ok(Self {
                    #(#readers,)*
                })
            }

            fn fields() -> &'static [::macros_in_depth::FieldInfo] {
                const FIELDS: &[::macros_in_depth::FieldInfo] = &[#(#infos),*];
                FIELDS
            }
        }
    })
}

fn push_error(errors: &mut Option<Error>, error: Error) {
    match errors {
        Some(existing) => existing.combine(error),
        None => *errors = Some(error),
    }
}

/// Reads one field's name, type, doc comment and `#[setting(…)]` attribute.
fn plan_field(field: &Field) -> syn::Result<FieldPlan<'_>> {
    let ident = field.ident.as_ref().expect("named fields have names");
    let mut plan = FieldPlan {
        field,
        key: ident.to_string(),
        default: None,
        doc: String::new(),
        optional: option_inner(&field.ty).is_some(),
    };

    for attr in &field.attrs {
        // `/// text` arrives as `#[doc = " text"]`.
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(pair) = &attr.meta
                && let Expr::Lit(expr) = &pair.value
                && let Lit::Str(text) = &expr.lit
            {
                if !plan.doc.is_empty() {
                    plan.doc.push(' ');
                }
                plan.doc.push_str(text.value().trim());
            }
            continue;
        }
        if !attr.path().is_ident("setting") {
            continue;
        }
        // `#[setting(default = "8080", rename = "listen-port")]`
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("default") {
                let value: Lit = meta.value()?.parse()?;
                let Lit::Str(text) = value else {
                    return Err(Error::new_spanned(
                        value,
                        "write the default as it would appear in the settings file, in quotes: `default = \"8080\"`",
                    ));
                };
                if plan.optional {
                    return Err(Error::new_spanned(
                        text,
                        "an `Option` field is already optional: it can't have a default",
                    ));
                }
                plan.default = Some(text);
                Ok(())
            } else if meta.path.is_ident("rename") {
                let text: LitStr = meta.value()?.parse()?;
                if text.value().is_empty() {
                    return Err(Error::new_spanned(text, "the key can't be empty"));
                }
                plan.key = text.value();
                Ok(())
            } else {
                // `meta.error` points at the unknown name itself.
                Err(meta.error("unknown setting option: expected `default` or `rename`"))
            }
        })?;
    }
    Ok(plan)
}

/// `Option<T>` gives `Some(T)`. (A type alias for `Option` isn't spotted:
/// a macro only sees tokens, never what a name refers to.)
fn option_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(path) = ty else { return None };
    let last = path.path.segments.last()?;
    if last.ident != "Option" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return None;
    };
    match args.args.first()? {
        syn::GenericArgument::Type(inner) if args.args.len() == 1 => Some(inner),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    fn expand_ok(input: DeriveInput) -> String {
        expand(&input).expect("should expand").to_string()
    }

    fn expand_err(input: DeriveInput) -> String {
        expand(&input).expect_err("should fail").to_string()
    }

    #[test]
    fn generates_a_reader_per_field() {
        let out = expand_ok(parse_quote! {
            struct Server {
                /// The port.
                #[setting(default = "8080")]
                port: u16,
                #[setting(rename = "log-file")]
                log: Option<String>,
            }
        });
        assert!(out.contains("impl :: macros_in_depth :: Settings for Server"));
        assert!(out.contains(
            "required (map , \"port\" , :: core :: option :: Option :: Some (\"8080\"))"
        ));
        assert!(
            out.contains("optional (map , \"log-file\" , :: core :: option :: Option :: None)")
        );
        assert!(out.contains("doc : \"The port.\""));
        assert!(out.contains("type_name : \"Option<String>\""));
    }

    #[test]
    fn rejects_enums_tuple_structs_and_generics() {
        let enum_error = expand_err(parse_quote!(
            enum Mode {
                Fast,
            }
        ));
        assert!(enum_error.contains("only works on structs"));
        let tuple_error = expand_err(parse_quote!(
            struct Pair(u8, u8);
        ));
        assert!(tuple_error.contains("needs named fields"));
        let generic_error = expand_err(parse_quote!(
            struct Wrapper<T> {
                value: T,
            }
        ));
        assert!(generic_error.contains("generic parameters"));
    }

    #[test]
    fn explains_bad_setting_attributes() {
        let unknown = expand_err(parse_quote! {
            struct S { #[setting(defualt = "1")] a: u8 }
        });
        assert!(unknown.contains("unknown setting option"));
        let unquoted = expand_err(parse_quote! {
            struct S { #[setting(default = 1)] a: u8 }
        });
        assert!(unquoted.contains("in quotes"));
        let optional = expand_err(parse_quote! {
            struct S { #[setting(default = "x")] a: Option<String> }
        });
        assert!(optional.contains("already optional"));
    }

    #[test]
    fn reports_every_duplicate_key_at_once() {
        let error = expand(&parse_quote! {
            struct S {
                a: u8,
                #[setting(rename = "a")] b: u8,
                #[setting(rename = "a")] c: u8,
            }
        })
        .expect_err("duplicate keys");
        assert_eq!(error.into_iter().count(), 2, "two combined errors");
    }

    #[test]
    fn finds_the_type_inside_option() {
        let ty: Type = parse_quote!(Option<Vec<u8>>);
        let inner = option_inner(&ty).expect("an Option");
        assert_eq!(quote!(#inner).to_string(), "Vec < u8 >");
        assert!(option_inner(&parse_quote!(String)).is_none());
    }
}
