//! Recoverable errors with `Result<T, E>`.
//! Run: cargo run -p error_handling --example result_basics
//!
//! The standard library defines:
//!     enum Result<T, E> { Ok(T), Err(E) }
//! Like Option, `Ok` and `Err` are in scope without the `Result::` prefix.

use std::fs::{self, File};
use std::io::ErrorKind;

fn main() {
    // Work in the system temp folder so we never touch the repo.
    let path = std::env::temp_dir().join("rust_course_hello.txt");
    let _ = fs::remove_file(&path); // start fresh; ignore "not found" here

    // 1. match on the Result.
    match File::open(&path) {
        Ok(file) => println!("opened {file:?}"),
        Err(error) => println!("could not open: {error}"),
    }

    // 2. Different reactions for different kinds of error.
    let greeting_file = match File::open(&path) {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => match File::create(&path) {
                Ok(fc) => {
                    println!("not found, so created it");
                    fc
                }
                Err(e) => panic!("Problem creating the file: {e:?}"),
            },
            other_error => panic!("Problem opening the file: {other_error:?}"),
        },
    };
    drop(greeting_file);

    // 3. The same logic, flatter, with closures (lesson 03/04).
    let _file = File::open(&path).unwrap_or_else(|error| {
        if error.kind() == ErrorKind::NotFound {
            File::create(&path).unwrap_or_else(|error| {
                panic!("Problem creating the file: {error:?}");
            })
        } else {
            panic!("Problem opening the file: {error:?}");
        }
    });
    println!("opened it again");

    // 4. Shortcuts that PANIC on Err. Fine in tests, examples and prototypes.
    //    `expect` is preferred: its message says what you assumed.
    let text = fs::read_to_string(&path).expect("hello.txt should be readable");
    println!("file holds {} bytes", text.len());
    // File::open("no/such/file.txt").unwrap();
    // ^ panics: called `Result::unwrap()` on an `Err` value: Os { kind: NotFound, .. }

    // 5. Result has methods like Option's: is_ok, ok(), map, unwrap_or ...
    let parsed: Result<i32, _> = "42".parse::<i32>();
    let bad: Result<i32, _> = "forty-two".parse::<i32>();
    println!("{parsed:?} {bad:?}");
    println!("with a default: {}", bad.clone().unwrap_or(0));
    println!("as an Option: {:?}", bad.ok());

    fs::remove_file(&path).expect("clean up the temp file");
}
