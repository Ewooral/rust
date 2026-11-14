//! Shell completions with `clap_complete`: clap already knows every
//! subcommand, option and possible value, so it can write the script that
//! makes Tab complete them.
//! Run: cargo run -p clap_cli --example completions
//!
//! The usual way to ship this is a `completions <SHELL>` subcommand that
//! writes the script to stdout, as the mini project does. To install one:
//!
//!   bash:        source <(clap_cli completions bash)       (add it to ~/.bashrc)
//!   PowerShell:  clap_cli completions powershell | Out-String | Invoke-Expression
//!                (add that line to your $PROFILE)
//!   zsh, fish, elvish: see `clap_cli completions --help` for the shells.

use clap::CommandFactory;
use clap_complete::{Shell, generate};

fn main() {
    let mut cmd = clap_cli::Cli::command();
    for shell in [
        Shell::Bash,
        Shell::Zsh,
        Shell::Fish,
        Shell::PowerShell,
        Shell::Elvish,
    ] {
        let mut script = Vec::new();
        generate(shell, &mut cmd, "clap_cli", &mut script);
        let text = String::from_utf8(script).unwrap();
        println!("{shell:<10} {:>5} lines", text.lines().count());
    }

    // What a completion script looks like: the start of the bash one.
    let mut script = Vec::new();
    generate(Shell::Bash, &mut cmd, "clap_cli", &mut script);
    println!("\nthe first lines of the bash script:");
    for line in String::from_utf8(script).unwrap().lines().take(8) {
        println!("  {line}");
    }
}
