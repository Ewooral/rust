//! WASI: an ordinary std program that can ALSO run as WebAssembly, outside a browser.
//! Run: cargo run -p webassembly --example wasi_program -- 05_mastery/09_webassembly/README.md
//!
//! WASI (the WebAssembly System Interface) gives a module the operating
//! system parts of std: args, environment variables, stdin/stdout, the
//! clock, and files in folders you allow. So this file uses plain std and no
//! wasm-bindgen at all. The same source builds for a WASI target unchanged
//! (NOT run here: the target and wasmtime aren't installed):
//!
//!   rustup target add wasm32-wasip2
//!   cargo new wasi-demo && cp wasi_program.rs wasi-demo/src/main.rs
//!   cargo build --target wasm32-wasip2 --release        (in wasi-demo/)
//!   wasmtime run --dir=. target/wasm32-wasip2/release/wasi-demo.wasm README.md
//!
//! `--dir=.` is the sandbox: without it, the module can't open any file at
//! all. Environment variables are empty too unless you pass `--env`.

use std::io::Read;
use std::time::Instant;

fn count(text: &str) -> (usize, usize, usize) {
    (
        text.lines().count(),
        text.split_whitespace().count(),
        text.chars().count(),
    )
}

fn main() {
    let started = Instant::now(); // WASI provides a clock
    let args: Vec<String> = std::env::args().skip(1).collect();

    let inputs: Vec<(String, Result<String, std::io::Error>)> = if args.is_empty() {
        println!("(no file given: reading stdin; try passing a file name)");
        let mut text = String::new();
        let result = std::io::stdin().read_to_string(&mut text).map(|_| text);
        vec![("<stdin>".to_string(), result)]
    } else {
        args.into_iter()
            .map(|path| {
                let text = std::fs::read_to_string(&path); // needs --dir under WASI
                (path, text)
            })
            .collect()
    };

    println!("{:>7} {:>7} {:>8}  file", "lines", "words", "chars");
    for (name, text) in inputs {
        match text {
            Ok(text) => {
                let (lines, words, chars) = count(&text);
                println!("{lines:>7} {words:>7} {chars:>8}  {name}");
            }
            // Under WASI without --dir, this is the error you'd see.
            Err(err) => println!("{:>24}  {name}: {err}", "error"),
        }
    }

    let user = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "(not set: under wasmtime, pass --env USER)".to_string());
    println!("\nuser: {user}");
    println!("took {:?}", started.elapsed());
}
