//! Passing a value to a function moves or copies it, exactly like assignment.
//! Returning a value moves ownership back out.
//! Run: cargo run -p ownership --example ownership_and_functions

fn takes_ownership(s: String) {
    println!("took {s}");
} // `s` is dropped here

fn makes_copy(n: i32) {
    println!("copied {n}");
}

fn gives_ownership() -> String {
    String::from("yours")
}

fn takes_and_gives_back(s: String) -> String {
    s
}

// The clumsy way: hand ownership in, get it back alongside the result.
fn calculate_length(s: String) -> (String, usize) {
    let len = s.len();
    (s, len)
}

fn main() {
    let s = String::from("hello");
    takes_ownership(s);
    // `s` is no longer valid here.

    let n = 5;
    makes_copy(n);
    println!("n is still {n}");

    let s1 = gives_ownership();
    let s2 = takes_and_gives_back(s1);
    let (s3, len) = calculate_length(s2);
    println!("'{s3}' has length {len}");
    // Passing ownership back and forth is tedious. References fix this.
}
