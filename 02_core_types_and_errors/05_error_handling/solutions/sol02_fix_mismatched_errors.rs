//! Solution to exercise 2: make it compile (E0277).
//!
//! `read_number` can fail in two DIFFERENT ways: reading the file (io::Error)
//! and parsing the text (ParseIntError). Its return type only allows one.
//! Fix it by changing the error type in the signature to one that every
//! error can convert into. Do not change the body.
//!
//! Run: cargo run -p error_handling --features solutions --bin sol02_fix_mismatched_errors

use std::error::Error;
use std::fs;
use std::path::Path;

// Box<dyn Error>: both io::Error and ParseIntError convert into it.
fn read_number(path: &Path) -> Result<i64, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let n = text.trim().parse()?;
    Ok(n)
}

fn main() {
    let path = std::env::temp_dir().join("rust_course_number.txt");
    fs::write(&path, "1234\n").expect("write temp file");
    println!("{:?}", read_number(&path));
    println!("{:?}", read_number(Path::new("missing.txt")).is_err());
    fs::remove_file(&path).expect("clean up");
}
