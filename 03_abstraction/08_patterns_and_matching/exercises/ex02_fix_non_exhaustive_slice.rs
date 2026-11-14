//! Exercise 2: make it compile (E0004).
//!
//! `who_is_coming` handles a list of 0, 1 or 2 names, but a slice can be any
//! length, so the match isn't exhaustive. Add ONE arm, without `_`, for
//! three or more names, that produces e.g. "Ama, Kofi and 2 others".
//! Hint: `[first, second, rest @ ..]` binds the rest as a slice.
//!
//! Run: cargo run -p patterns_and_matching --features exercises --bin ex02_fix_non_exhaustive_slice

fn who_is_coming(names: &[&str]) -> String {
    match names {
        [] => "nobody".to_string(),
        [one] => format!("just {one}"),
        [first, second] => format!("{first} and {second}"),
    }
}

fn main() {
    let lists: [&[&str]; 4] = [
        &[],
        &["Ama"],
        &["Ama", "Kofi"],
        &["Ama", "Kofi", "Esi", "Yaw"],
    ];
    for names in lists {
        println!("{}", who_is_coming(names));
    }
}
