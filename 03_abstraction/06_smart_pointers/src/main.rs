//! Mini project for Book chapter 15: tinyfs, a pretend file system made of
//! smart pointers (all the logic is in src/lib.rs and its modules).
//!
//!   cargo run -p smart_pointers              a scripted demo session
//!   cargo run -p smart_pointers -- -         type your own commands
//!                                            (end with Ctrl+Z then Enter on
//!                                            Windows, Ctrl+D on Linux)
//!   cargo test -p smart_pointers

use std::io::{self, BufRead, Write};

use smart_pointers::Shell;

const DEMO: &[&str] = &[
    "help",
    "mkdir projects",
    "mkdir projects/rust",
    "cd projects/rust",
    "pwd",
    "write notes.txt Box owns one value on the heap.",
    "write notes.txt Rc counts owners. Weak does not.",
    "cat notes.txt",
    "ln notes.txt /",
    "stat notes.txt",
    "cd ../..",
    "ls",
    "tree",
    "stat projects",
    "rm projects",
    "stat notes.txt",
    "cat notes.txt",
    "cd projects",
    "history",
];

fn main() -> io::Result<()> {
    let mut shell = Shell::new();
    if std::env::args().nth(1).as_deref() == Some("-") {
        let stdin = io::stdin();
        prompt(&shell)?;
        for line in stdin.lock().lines() {
            show(shell.run(&line?));
            prompt(&shell)?;
        }
        println!();
    } else {
        for line in DEMO {
            println!("tinyfs:{}$ {line}", shell.pwd());
            show(shell.run(line));
        }
    }

    // Dropping the shell drops the root and the current directory. Because
    // parents are only Weak links, nothing keeps anything else alive.
    let log = shell.drop_log();
    let freed_before = log.borrow().len();
    println!("\n(the shell is dropped)");
    drop(shell);
    for line in &log.borrow()[freed_before..] {
        println!("{line}");
    }
    Ok(())
}

fn prompt(shell: &Shell) -> io::Result<()> {
    print!("tinyfs:{}$ ", shell.pwd());
    io::stdout().flush()
}

fn show(result: Result<String, String>) {
    match result {
        Ok(output) if output.is_empty() => {}
        Ok(output) => println!("{output}"),
        Err(message) => println!("error: {message}"),
    }
}
