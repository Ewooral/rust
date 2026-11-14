//! Allocation awareness: count what your code asks of the heap.
//!
//! Run: cargo run -p performance --example allocations
//!
//! Allocation counts don't depend on the build or on what else the machine
//! is doing, so this example prints the same numbers in debug and release.

use std::fmt::Write as _;

use performance::alloc_counter::{CountingAlloc, count_allocations};

// Install the counting allocator for this program (src/alloc_counter.rs).
#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

fn report(what: &str, f: impl FnOnce()) {
    let ((), stats) = count_allocations(f);
    println!(
        "{what:<48} {:>5} allocs {:>8} bytes",
        stats.allocations, stats.bytes
    );
}

fn total_len_owned(words: Vec<String>) -> usize {
    words.iter().map(String::len).sum()
}

fn total_len_borrowed(words: &[String]) -> usize {
    words.iter().map(String::len).sum()
}

fn shout_owned(text: String) -> usize {
    text.len()
}

fn shout_borrowed(text: &str) -> usize {
    text.len()
}

fn main() {
    println!("1. Growing a Vec");
    report("   push 10 000 items into Vec::new()", || {
        let mut v = Vec::new();
        for i in 0..10_000u32 {
            v.push(i);
        }
    });
    report("   push 10 000 items into with_capacity(10 000)", || {
        let mut v = Vec::with_capacity(10_000);
        for i in 0..10_000u32 {
            v.push(i);
        }
    });
    report("   (0..10 000).collect() (it knows its size)", || {
        let _v: Vec<u32> = (0..10_000).collect();
    });

    println!("\n2. Building a String");
    report("   s = s + &i.to_string(), 1 000 times", || {
        let mut s = String::new();
        for i in 0..1_000 {
            s = s + &i.to_string(); // to_string allocates every time too
        }
    });
    report("   write!(s, \"{i}\") into with_capacity(4 000)", || {
        let mut s = String::with_capacity(4_000);
        for i in 0..1_000 {
            let _ = write!(s, "{i}");
        }
    });

    let words: Vec<String> = (0..1_000).map(|i| format!("word{i}")).collect();
    println!("\n3. Cloning vs borrowing ({} words)", words.len());
    report("   total_len_owned(words.clone())", || {
        total_len_owned(words.clone());
    });
    report("   total_len_borrowed(&words)", || {
        total_len_borrowed(&words);
    });

    println!("\n4. String vs &str parameters");
    report("   shout_owned(\"hello\".to_string())", || {
        shout_owned("hello".to_string());
    });
    report("   shout_borrowed(\"hello\")", || {
        shout_borrowed("hello");
    });
}
