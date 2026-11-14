//! Reading a line from the keyboard with `stdin().read_line`.
//! Run: cargo run -p guessing_game --example read_input
//! Or pipe a line in: echo hello | cargo run -p guessing_game --example read_input
//!
//! `{:?}` (Debug formatting) shows the invisible characters, so you can see
//! the newline that Enter adds: `"\n"` on Linux, `"\r\n"` on Windows.

use std::io;

fn main() {
    println!("Type something and press Enter:");

    // `String::new()` makes an empty, growable string. It must be `mut`
    // because `read_line` is going to append to it.
    let mut line = String::new();

    // `&mut line` lends the string to `read_line` so it can change it.
    // `read_line` returns a `Result`: `Ok(bytes_read)` or `Err(io_error)`.
    // `expect` gives us the `Ok` value, or crashes with our message.
    let bytes_read = io::stdin()
        .read_line(&mut line)
        .expect("Failed to read line");

    if bytes_read == 0 {
        println!("No input at all (end of input).");
        return;
    }

    println!("read {bytes_read} bytes");
    println!("exactly as typed: {line:?}");
    println!("after trim():     {:?}", line.trim());
}
