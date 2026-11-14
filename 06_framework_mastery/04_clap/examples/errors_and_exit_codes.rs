//! Errors and exit codes: what `try_parse_from` gives back, which kinds of
//! "error" are really help or version requests, how clap picks the exit
//! code, and raising your own clap-style error after parsing.
//! Run: cargo run -p clap_cli --example errors_and_exit_codes
//!
//! `Cli::parse()` handles all of this for you: it prints the message and
//! calls `std::process::exit` with the right code. `try_parse_from` lets you
//! look first, which is what tests (and this example) want.

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};

#[derive(Parser, Debug)]
#[command(name = "convert", version = "1.0.0")]
struct Convert {
    /// Input file
    input: String,
    /// Output file
    #[arg(short, long)]
    output: Option<String>,
    /// Quality, 1 to 100
    #[arg(short, long, default_value_t = 90, value_parser = clap::value_parser!(u8).range(1..=100))]
    quality: u8,
}

fn main() {
    let cases: [&[&str]; 6] = [
        &["convert", "a.png"],
        &["convert", "--help"],
        &["convert", "--version"],
        &["convert"],
        &["convert", "a.png", "--quality", "300"],
        &["convert", "a.png", "--colour"],
    ];
    for args in cases {
        print!("{:<40}", args.join(" "));
        match Convert::try_parse_from(args) {
            Ok(_) => println!("Ok                          exit code 0 (your program decides)"),
            Err(e) => println!(
                "{:<36} exit code {}, printed to {}",
                format!("{:?}", e.kind()),
                e.exit_code(),
                if e.use_stderr() { "stderr" } else { "stdout" }
            ),
        }
    }

    // A rule clap can't check by itself: the output must differ from the
    // input. `Command::error` makes an error that looks like clap's own.
    let args = Convert::try_parse_from(["convert", "a.png", "-o", "a.png"]).unwrap();
    if args.output.as_deref() == Some(args.input.as_str()) {
        let err = Convert::command().error(
            ErrorKind::ArgumentConflict,
            "the output file must be different from the input file",
        );
        println!(
            "\nyour own error, in clap's style (exit code {}):",
            err.exit_code()
        );
        print!("{}", err.render());
        // In a real main: err.exit();
    }
}
