//! Exercise 5: slice patterns (Beyond the Book).
//!
//! Write each function with a `match` on the slice, not with indexing or
//! `.len()`:
//!
//! 1. `ends`: the first and last numbers. One number is both ends; an empty
//!    slice has none.
//! 2. `is_sorted`: true if every number is <= the next one. Use recursion:
//!    look at the first TWO numbers, then call yourself on the rest.
//! 3. `route`: a tiny web router. The path "/users/7/posts" arrives already
//!    split into `["users", "7", "posts"]`:
//!    - `[]` -> "home"
//!    - `["users"]` -> "all users"
//!    - `["users", id]` -> "user 7"
//!    - `["users", id, "posts"]` -> "posts by user 7"
//!    - `["static", file @ ..]` with at least one part -> "file css/site.css"
//!    - anything else -> "404 not found: a/b"
//!
//! Test: cargo test -p patterns_and_matching --features exercises --bin ex05_slice_patterns

#![allow(unused_variables)]

fn ends(numbers: &[i32]) -> Option<(i32, i32)> {
    todo!()
}

fn is_sorted(numbers: &[i32]) -> bool {
    todo!()
}

fn route(path: &[&str]) -> String {
    todo!()
}

fn main() {
    println!("{:?}", ends(&[3, 1, 4, 1, 5]));
    println!("{}", is_sorted(&[1, 2, 2, 9]));
    for url in [
        "/",
        "/users",
        "/users/7/posts",
        "/static/css/site.css",
        "/admin",
    ] {
        let parts: Vec<&str> = url.split('/').filter(|p| !p.is_empty()).collect();
        println!("{url} -> {}", route(&parts));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_and_last() {
        assert_eq!(ends(&[]), None);
        assert_eq!(ends(&[7]), Some((7, 7)));
        assert_eq!(ends(&[3, 1, 4, 1, 5]), Some((3, 5)));
    }

    #[test]
    fn sorted_or_not() {
        assert!(is_sorted(&[]));
        assert!(is_sorted(&[42]));
        assert!(is_sorted(&[1, 2, 2, 9]));
        assert!(!is_sorted(&[1, 3, 2]));
        assert!(!is_sorted(&[5, 1, 2, 3]));
    }

    #[test]
    fn routes() {
        assert_eq!(route(&[]), "home");
        assert_eq!(route(&["users"]), "all users");
        assert_eq!(route(&["users", "7"]), "user 7");
        assert_eq!(route(&["users", "7", "posts"]), "posts by user 7");
        assert_eq!(route(&["static", "css", "site.css"]), "file css/site.css");
        assert_eq!(route(&["static"]), "404 not found: static");
        assert_eq!(
            route(&["users", "7", "likes"]),
            "404 not found: users/7/likes"
        );
    }
}
