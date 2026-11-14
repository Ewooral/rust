//! Exercise 1: make it compile (E0106).
//!
//! The compiler can't tell whether the returned lines borrow from `query` or
//! from `contents`. They come from `contents` only. Add ONE lifetime
//! parameter, on `contents` and the return type, and NOT on `query`.
//! (Why would putting it on `query` too be needlessly strict?)
//!
//! Run: cargo run -p minigrep --features exercises --bin ex01_fix_search_lifetime

fn search(query: &str, contents: &str) -> Vec<&str> {
    let mut results = Vec::new();
    for line in contents.lines() {
        if line.contains(query) {
            results.push(line);
        }
    }
    results
}

fn main() {
    let poem = "I'm nobody! Who are you?\nAre you nobody, too?";
    let found = {
        let query = String::from("nobody"); // dropped at the end of this block
        search(&query, poem)
    };
    println!("{found:?}");
}
