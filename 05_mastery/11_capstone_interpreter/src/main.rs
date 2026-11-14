//! kite: run a Kite program, or start the REPL.
//!
//! From this folder (05_mastery/11_capstone_interpreter):
//!   cargo run -- programs/fib.kite            run a script
//!   cargo run                                 the REPL (Ctrl+D, or Ctrl+Z then Enter on Windows, to quit)
//!   echo 'print(1 + 2);' | cargo run          the REPL, fed from a pipe
//!
//! Exit codes, from the Unix sysexits.h conventions (Crafting Interpreters
//! uses 64, 65 and 70 the same way):
//!   0 ok, 64 usage, 65 syntax error, 66 cannot open the file, 70 runtime error.

use std::env;
use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::process::ExitCode;
use std::thread;

use capstone_interpreter::{ErrorKind, Event, Interpreter, Repl};

/// Each Kite call is several Rust calls, so deep Kite recursion needs a
/// deep Rust stack. The main thread's stack is small (1 MiB on Windows),
/// so the interpreter runs on a thread with a bigger one. The operating
/// system only reserves this address space; memory is used as it's touched.
const STACK_SIZE: usize = 64 * 1024 * 1024;

fn main() -> ExitCode {
    let worker = thread::Builder::new()
        .name("kite".into())
        .stack_size(STACK_SIZE)
        .spawn(run)
        .expect("can start the interpreter thread");
    // A panic in the interpreter is a bug in it: report it as a failure.
    worker.join().unwrap_or(ExitCode::from(70))
}

fn run() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [] => repl(),
        [path] => run_file(path),
        _ => {
            eprintln!("usage: kite [script.kite]");
            ExitCode::from(64)
        }
    }
}

fn run_file(path: &str) -> ExitCode {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("kite: cannot read {path}: {e}");
            return ExitCode::from(66);
        }
    };
    let mut kite = Interpreter::new(io::stdout().lock());
    match kite.run(&source) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", error.render(&source));
            match error.kind {
                ErrorKind::Syntax => ExitCode::from(65),
                ErrorKind::Runtime => ExitCode::from(70),
            }
        }
    }
}

fn repl() -> ExitCode {
    // Prompts are for people: don't print them into a pipe (lesson 05/10).
    let interactive = io::stdin().is_terminal();
    if interactive {
        println!("Kite REPL. Ctrl+D (Ctrl+Z then Enter on Windows) to quit.");
    }
    let mut repl = Repl::new(io::stdout());
    let mut failed = false;
    let mut lines = io::stdin().lock().lines();

    loop {
        if interactive {
            prompt(if repl.is_continuing() { "... " } else { "> " });
        }
        // End of input (Ctrl+D / Ctrl+Z, or the end of a pipe) ends the REPL.
        let Some(Ok(line)) = lines.next() else { break };
        match repl.feed(&line) {
            Event::NeedMore | Event::Done(None) => {}
            Event::Done(Some(value)) => println!("{value:?}"),
            Event::Error(text) => {
                eprintln!("{text}");
                failed = true;
            }
        }
    }
    // Piped input that hit an error should say so to the script that piped it.
    if failed && !interactive {
        ExitCode::from(70)
    } else {
        ExitCode::SUCCESS
    }
}

fn prompt(text: &str) {
    print!("{text}");
    let _ = io::stdout().flush(); // no newline, so flush by hand
}
