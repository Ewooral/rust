//! Unrecoverable errors with `panic!`. THIS EXAMPLE PANICS ON PURPOSE.
//!
//! Run:                cargo run -p error_handling --example panic
//! Explicit panic:     cargo run -p error_handling --example panic -- explicit
//! With a backtrace:   set RUST_BACKTRACE=1 first
//!   PowerShell:  $env:RUST_BACKTRACE=1; cargo run -p error_handling --example panic
//!   bash:        RUST_BACKTRACE=1 cargo run -p error_handling --example panic

fn main() {
    let explicit = std::env::args().any(|a| a == "explicit");

    if explicit {
        // 1. Calling panic! yourself: the program prints the message,
        //    unwinds the stack (dropping everything) and exits with an error.
        panic!("crash and burn");
    }

    // 2. A bug in our code makes a library panic: indexing past the end.
    //    In C this would read whatever memory is there (a buffer over-read);
    //    Rust stops instead.
    #[allow(clippy::useless_vec)] // the Book uses a Vec here; an array would fail at compile time
    let v = vec![1, 2, 3];
    let index = v.len() + 96; // pretend this came from a miscalculation
    println!("about to read v[{index}]...");
    let value = v[index]; // panics: index out of bounds: the len is 3 but the index is 99
    println!("never printed: {value}");
}

// Unwinding (the default) walks back up the stack and cleans up. To make the
// final binary smaller, a project can choose to ABORT instead, in Cargo.toml:
//     [profile.release]
//     panic = 'abort'
