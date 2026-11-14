//! The builder API: the same kind of command line as `derive_basics`, built
//! from `Command` and `Arg` values at run time, and read back by name.
//! Run: cargo run -p clap_cli --example builder_api
//!
//! The derive API generates exactly this kind of code. Reach for the
//! builder when the command line isn't known at compile time (plugins,
//! arguments read from a config file) or to tweak a derived `Command`
//! (`Cli::command().mut_arg(..)`).

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};

fn command() -> Command {
    Command::new("greet")
        .version("1.0")
        .about("Greets someone, as many times as you like")
        .arg(Arg::new("name").required(true).help("Who to greet"))
        .arg(
            Arg::new("times")
                .short('t')
                .long("times")
                .help("How many times")
                .value_parser(value_parser!(u8))
                .default_value("1"),
        )
        .arg(
            Arg::new("shout")
                .short('s')
                .long("shout")
                .help("Use capital letters")
                .action(ArgAction::SetTrue),
        )
}

fn greet(m: &ArgMatches) {
    // Values come back by NAME and TYPE: a typo or a wrong type is a panic
    // at run time, not a compile error. That's what the derive API saves you.
    let name = m.get_one::<String>("name").expect("required");
    let times = *m.get_one::<u8>("times").expect("has a default");
    let shout = m.get_flag("shout");
    for _ in 0..times {
        let line = format!("Hello, {name}!");
        println!("    {}", if shout { line.to_uppercase() } else { line });
    }
}

fn main() {
    for args in [
        &["greet", "Ada"][..],
        &["greet", "Ada", "-t", "2", "--shout"],
        &["greet", "-t", "x", "Ada"],
    ] {
        println!("$ {}", args.join(" "));
        match command().try_get_matches_from(args) {
            Ok(m) => greet(&m),
            Err(e) => print!("{}", e.render()),
        }
    }
    println!("$ greet --help");
    print!("{}", command().render_help());
}
