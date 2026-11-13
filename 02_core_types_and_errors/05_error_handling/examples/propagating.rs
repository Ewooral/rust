//! Propagating errors to the caller, by hand and with `?`.
//! Run: cargo run -p error_handling --example propagating

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

/// Version 1: by hand. Every failure point is a `match` that returns Err.
// Clippy suggests `?` here, which is exactly the point of version 2.
#[allow(clippy::question_mark)]
fn read_username_long(path: &Path) -> Result<String, io::Error> {
    let username_file_result = File::open(path);
    let mut username_file = match username_file_result {
        Ok(file) => file,
        Err(e) => return Err(e),
    };
    let mut username = String::new();
    match username_file.read_to_string(&mut username) {
        Ok(_) => Ok(username),
        Err(e) => Err(e),
    }
}

/// Version 2: `?` after a Result means "if Err, return it from this function
/// right now; if Ok, give me the value".
fn read_username_question(path: &Path) -> Result<String, io::Error> {
    let mut username_file = File::open(path)?;
    let mut username = String::new();
    username_file.read_to_string(&mut username)?;
    Ok(username)
}

/// Version 3: chain the calls.
fn read_username_chained(path: &Path) -> Result<String, io::Error> {
    let mut username = String::new();
    File::open(path)?.read_to_string(&mut username)?;
    Ok(username)
}

/// Version 4: the standard library already has this function.
fn read_username_short(path: &Path) -> Result<String, io::Error> {
    fs::read_to_string(path)
}

/// `?` works on Option too, in a function that returns Option.
/// (You cannot mix: `?` on a Result in an Option function is an error.
/// Convert with `.ok()` or `.ok_or(...)`.)
fn last_char_of_first_line(text: &str) -> Option<char> {
    text.lines().next()?.chars().last()
}

fn main() {
    let path = std::env::temp_dir().join("rust_course_username.txt");
    fs::write(&path, "ferris\nsecond line").expect("write temp file");

    println!("{:?}", read_username_long(&path));
    println!("{:?}", read_username_question(&path));
    println!("{:?}", read_username_chained(&path));
    println!("{:?}", read_username_short(&path).map(|s| s.len()));

    let missing = Path::new("this/file/does/not/exist.txt");
    match read_username_question(missing) {
        Ok(name) => println!("got {name}"),
        Err(e) => println!("the caller decides what to do: {e} (kind {:?})", e.kind()),
    }

    println!(
        "{:?}",
        last_char_of_first_line("Hello, world\nHow are you?")
    );
    println!("{:?}", last_char_of_first_line(""));
    println!("{:?}", last_char_of_first_line("\nhi"));

    fs::remove_file(&path).expect("clean up");
}
