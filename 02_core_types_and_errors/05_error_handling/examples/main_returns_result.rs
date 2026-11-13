//! `main` can return a Result, so `?` works in it.
//!
//! Run:  cargo run -p error_handling --example main_returns_result -- 3 4 5
//! Fail: cargo run -p error_handling --example main_returns_result -- 3 four 5
//!
//! If main returns Err, Rust prints it with {:?} and the process exits with
//! a non-zero code, which scripts and CI can check.

use std::error::Error;

/// `Box<dyn Error>` means "any kind of error". Every error type converts
/// into it, so `?` works on io errors, parse errors and our own alike.
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args = if args.is_empty() {
        println!("(no numbers given, using 1 2 3)");
        vec!["1".to_string(), "2".to_string(), "3".to_string()]
    } else {
        args
    };

    let mut total: i64 = 0;
    for arg in &args {
        let n: i64 = arg.parse()?; // a ParseIntError becomes Box<dyn Error>
        total += n;
    }
    if total > 1_000 {
        return Err("total is too large".into()); // a &str converts too
    }
    println!("sum of {args:?} = {total}");
    Ok(())
}
