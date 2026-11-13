//! Mini project for Book chapter 8: a company directory with a text interface.
//!
//!   cargo run -p collections            runs a scripted demo
//!   cargo run -p collections -- -i      interactive: type commands yourself
//!
//! Commands: "Add <name> to <department>", "List <department>", "List all",
//! "quit". Test: cargo test -p collections

use std::io::{self, BufRead, Write};

use collections::directory::{Command, Directory};
use collections::{arrays, strings};

fn main() -> io::Result<()> {
    let mut directory = Directory::new();

    let interactive = std::env::args().any(|arg| arg == "-i");
    if !interactive {
        let script = [
            "Add Sally to Engineering",
            "Add Amir to Sales",
            "Add Bola to Engineering",
            "Add Mary Jane to Human Resources",
            "List Engineering",
            "Fire everyone",
            "List all",
        ];
        for line in script {
            println!("> {line}");
            respond(&mut directory, line);
        }

        // Your earlier helpers, now part of the same library.
        println!("\nlibrary helpers:");
        println!("  {:?}", strings::group_in_pairs("abcdefghi"));
        println!("  index_of 3 = {:?}", arrays::index_of(&[1, 2, 3, 4], 3));
        println!("  sum = {}", arrays::sum(&[1, 2, 3, 4]));
        return Ok(());
    }

    println!("Company directory. Type a command, or 'quit'.");
    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush()?; // print! does not flush on its own
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 || line.trim() == "quit" {
            break; // 0 bytes read means end of input (Ctrl+Z / Ctrl+D)
        }
        respond(&mut directory, &line);
    }
    Ok(())
}

fn respond(directory: &mut Directory, line: &str) {
    match Command::parse(line) {
        Some(command) => println!("{}", directory.run(command)),
        None => println!("Sorry, I don't understand {:?}.", line.trim()),
    }
}
