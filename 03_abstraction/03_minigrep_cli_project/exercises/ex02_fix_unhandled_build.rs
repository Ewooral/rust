//! Exercise 2: make it compile (E0609).
//!
//! `Config::build` returns a Result, but `main` treats it as a Config.
//! Handle it the way minigrep does: on Err, print the message to STDERR
//! (`eprintln!`) and exit with code 1. Use `unwrap_or_else` and
//! `std::process::exit`. Then run it with no arguments, and with two.
//!
//! Run: cargo run -p minigrep --features exercises --bin ex02_fix_unhandled_build -- to poem.txt

struct Config {
    query: String,
    file_path: String,
}

impl Config {
    fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }
        Ok(Config {
            query: args[1].clone(),
            file_path: args[2].clone(),
        })
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config = Config::build(&args);
    println!("Searching for {} in {}", config.query, config.file_path);
}
