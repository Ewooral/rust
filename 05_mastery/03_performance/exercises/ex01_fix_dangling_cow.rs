//! Exercise 1: make it compile (E0515), without giving up the speed-up.
//!
//! `normalise` lowercases an HTTP header name: "Content-Type" becomes
//! "content-type". Almost every name in our logs is ALREADY lowercase, so
//! copying it into a new String is wasted work. The author tried to save
//! that copy by returning `&str`... and the compiler refused.
//!
//! Fix `normalise` with `std::borrow::Cow<'_, str>`: return
//! `Cow::Borrowed(name)` when there is nothing to change, and only allocate
//! (`Cow::Owned(...)`) when the name really has capital letters.
//! `main` and the tests don't need to change: a `Cow<str>` prints like a
//! `&str` and compares with one.
//!
//! Test: cargo test -p performance --features exercises --bin ex01_fix_dangling_cow

use performance::alloc_counter::CountingAlloc;

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

fn normalise(name: &str) -> &str {
    let lower = name.to_ascii_lowercase();
    &lower
}

fn main() {
    for name in [
        "content-type",
        "Content-Type",
        "x-request-id",
        "X-Request-ID",
    ] {
        println!("{name:>14} -> {}", normalise(name));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use performance::alloc_counter::count_allocations;

    #[test]
    fn lowercases_names_with_capitals() {
        assert_eq!(normalise("Content-Type"), "content-type");
        assert_eq!(normalise("X-Request-ID"), "x-request-id");
    }

    #[test]
    fn leaves_lowercase_names_alone() {
        assert_eq!(normalise("accept"), "accept");
    }

    #[test]
    fn the_common_case_allocates_nothing() {
        let (len, stats) = count_allocations(|| normalise("content-length").len());
        assert_eq!(len, 14);
        assert_eq!(stats.allocations, 0, "an already-lowercase name was copied");
    }

    #[test]
    fn the_rare_case_allocates_once() {
        let (_, stats) = count_allocations(|| normalise("Content-Length").len());
        assert_eq!(stats.allocations, 1);
    }
}
