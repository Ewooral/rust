//! The derive API: a struct with `#[derive(Parser)]` IS the command line.
//! Fields become arguments, doc comments become the help text.
//! Run: cargo run -p clap_cli --example derive_basics
//!      cargo run -p clap_cli --example derive_basics -- Ada --times 2 --shout
//!
//! With no arguments it parses a few made-up command lines with
//! `try_parse_from`, which returns a `Result` instead of exiting.

use clap::{CommandFactory, Parser};

/// Greets someone, as many times as you like
#[derive(Parser, Debug)]
#[command(name = "greet", version, about)]
struct Greet {
    /// Who to greet
    name: String, // no #[arg(..)]: a POSITIONAL argument, required

    /// How many times
    #[arg(short, long, default_value_t = 1)]
    times: u8, // an OPTION with a value: -t 3, --times 3, --times=3

    /// Use capital letters
    #[arg(short, long)]
    shout: bool, // a FLAG: present = true

    /// Sign the greeting
    #[arg(long)]
    from: Option<String>, // Option: may be left out

    /// Also greet these people
    #[arg(long = "and", value_name = "NAME")]
    others: Vec<String>, // Vec: may be repeated: --and Bo --and Cy
}

fn greet(g: &Greet) {
    let mut people = vec![g.name.clone()];
    people.extend(g.others.iter().cloned());
    for _ in 0..g.times {
        let line = format!("Hello, {}!", people.join(" and "));
        println!("    {}", if g.shout { line.to_uppercase() } else { line });
    }
    if let Some(from) = &g.from {
        println!("    -- {from}");
    }
}

fn show(args: &[&str]) {
    println!("$ {}", args.join(" "));
    match Greet::try_parse_from(args) {
        Ok(g) => {
            println!("  {g:?}");
            greet(&g);
        }
        Err(e) => print!("{}", e.render()),
    }
}

fn main() {
    if std::env::args().len() > 1 {
        let g = Greet::parse(); // the real arguments; exits on bad input
        greet(&g);
        return;
    }
    show(&["greet", "Ada"]);
    show(&["greet", "Ada", "--times", "2", "-s"]);
    show(&["greet", "Ada", "--and", "Bo", "--and", "Cy", "--from=Dad"]);
    show(&["greet", "--times", "two", "Ada"]);
    show(&["greet"]);
    println!("$ greet --help");
    print!("{}", Greet::command().render_help());
}
