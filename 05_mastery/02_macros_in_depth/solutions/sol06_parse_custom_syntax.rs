//! Solution to exercise 6: give a function-like macro its own syntax (pass 3 tests).
//!
//! A `routes!` macro would accept a little routing table:
//!
//!     routes! {
//!         GET "/users" => list_users,
//!         POST "/users" => create_user,
//!     }
//!
//! Its first job is to parse that. Implement `syn::parse::Parse`:
//!
//!   Route:  a method (`Ident`), a path (`LitStr`), `=>`, a handler (`Ident`).
//!     - The method must be GET, POST, PUT or DELETE. Otherwise return
//!       `Err(syn::Error::new(method.span(), format!(…)))` with the message
//!       "unknown HTTP method `FETCH`: expected GET, POST, PUT or DELETE".
//!     - The path must start with '/'. Otherwise: an error on the literal,
//!       "a route path must start with `/`".
//!     - `input.parse::<Token![=>]>()?` consumes the arrow (or reports
//!       "expected `=>`" by itself).
//!   Routes: zero or more routes separated by commas, trailing comma
//!     allowed: `Punctuated::<Route, Token![,]>::parse_terminated(input)?`.
//!
//! Test: cargo test -p macros_in_depth --features solutions --bin sol06_parse_custom_syntax

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Ident, LitStr, Token};

struct Route {
    method: Ident,
    path: LitStr,
    handler: Ident,
}

struct Routes {
    routes: Vec<Route>,
}

impl Parse for Route {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let method: Ident = input.parse()?;
        if !["GET", "POST", "PUT", "DELETE"].contains(&method.to_string().as_str()) {
            return Err(syn::Error::new(
                method.span(),
                format!("unknown HTTP method `{method}`: expected GET, POST, PUT or DELETE"),
            ));
        }
        let path: LitStr = input.parse()?;
        if !path.value().starts_with('/') {
            return Err(syn::Error::new(
                path.span(),
                "a route path must start with `/`",
            ));
        }
        input.parse::<Token![=>]>()?;
        let handler: Ident = input.parse()?;
        Ok(Route {
            method,
            path,
            handler,
        })
    }
}

impl Parse for Routes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let routes = Punctuated::<Route, Token![,]>::parse_terminated(input)?;
        Ok(Routes {
            routes: routes.into_iter().collect(),
        })
    }
}

fn main() {
    let table = r#"
        GET "/users" => list_users,
        POST "/users" => create_user,
        DELETE "/users/:id" => delete_user,
    "#;
    match syn::parse_str::<Routes>(table) {
        Ok(parsed) => {
            for route in parsed.routes {
                println!(
                    "{:6} {:12} -> {}",
                    route.method.to_string(), // an Ident ignores {:6} padding
                    route.path.value(),
                    route.handler
                );
            }
        }
        Err(error) => println!("error: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(source: &str) -> syn::Result<Vec<String>> {
        let parsed: Routes = syn::parse_str(source)?;
        Ok(parsed
            .routes
            .iter()
            .map(|r| format!("{} {} {}", r.method, r.path.value(), r.handler))
            .collect())
    }

    #[test]
    fn parses_a_table() {
        let routes = summary(r#"GET "/" => home, PUT "/items/:id" => replace_item,"#).unwrap();
        assert_eq!(routes, ["GET / home", "PUT /items/:id replace_item"]);
        assert!(summary("").unwrap().is_empty(), "an empty table is fine");
    }

    #[test]
    fn rejects_bad_methods_and_paths() {
        let method = summary(r#"FETCH "/" => home"#).unwrap_err();
        assert_eq!(
            method.to_string(),
            "unknown HTTP method `FETCH`: expected GET, POST, PUT or DELETE"
        );
        let path = summary(r#"GET "users" => home"#).unwrap_err();
        assert_eq!(path.to_string(), "a route path must start with `/`");
    }

    #[test]
    fn syntax_errors_come_from_syn() {
        let arrow = summary(r#"GET "/" -> home"#).unwrap_err();
        assert_eq!(arrow.to_string(), "expected `=>`");
        let comma = summary(r#"GET "/" => home GET "/a" => a"#).unwrap_err();
        assert_eq!(comma.to_string(), "expected `,`");
    }
}
