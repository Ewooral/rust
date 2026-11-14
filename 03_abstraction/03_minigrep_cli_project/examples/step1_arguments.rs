//! Step 1 (Book 12.1): accepting command line arguments.
//! Run: cargo run -p minigrep --example step1_arguments -- needle haystack.txt

use std::env;

fn main() {
    // env::args() is an ITERATOR over the arguments; collect() turns it into
    // a Vec. Collect needs to know the type, so we annotate Vec<String>.
    // (env::args panics if an argument is not valid Unicode; env::args_os
    // gives OsStrings instead.)
    let args: Vec<String> = env::args().collect();
    dbg!(&args);

    // args[0] is the program's own name. The user's arguments start at 1.
    // Indexing panics if they are missing: try running with no arguments.
    // (Step 3 replaces this with a proper error.)
    let query = &args[1];
    let file_path = &args[2];

    println!("Searching for {query}");
    println!("In file {file_path}");
}
