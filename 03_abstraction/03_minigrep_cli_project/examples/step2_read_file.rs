//! Step 2 (Book 12.2): reading a file.
//! From 03_abstraction/03_minigrep_cli_project:
//!   cargo run --example step2_read_file -- the poem.txt

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let query = &args[1];
    let file_path = &args[2];

    println!("Searching for {query}");
    println!("In file {file_path}");

    // Reads the WHOLE file into one String. Simple, and fine for files that
    // fit comfortably in memory (see the buffered_io example for big ones).
    let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");

    println!("With text:\n{contents}");

    // This main() now does four jobs (parse args, read, print, and soon
    // search) and panics on any problem. Step 3 fixes both.
}
