//! Help and version text: `about` and `long_about` from doc comments, `-h`
//! (short help) versus `--help` (long help), headings, hidden options,
//! text after the help, and the version string.
//! Run: cargo run -p clap_cli --example help_and_version

use clap::{CommandFactory, Parser};

/// Resize images.
///
/// Everything after the first paragraph of the doc comment is the LONG
/// about: shown by --help, but not by -h. Use it for details and caveats.
#[derive(Parser, Debug)]
#[command(name = "resize", version = "2.1.0", about)]
#[command(after_help = "Example: resize photo.jpg --width 800")]
struct Resize {
    /// The image to resize
    input: String,

    /// New width in pixels.
    ///
    /// The height follows, keeping the proportions, unless you give
    /// --height as well. (This paragraph only appears in --help.)
    #[arg(long, help_heading = "Size")]
    width: Option<u32>,

    /// New height in pixels
    #[arg(long, help_heading = "Size")]
    height: Option<u32>,

    /// Internal: dump timings (hidden from help, still accepted)
    #[arg(long, hide = true)]
    timings: bool,
}

fn main() {
    let mut cmd = Resize::command();
    println!("$ resize -h");
    print!("{}", cmd.render_help());
    println!("\n$ resize --help");
    print!("{}", cmd.render_long_help());
    println!("\n$ resize --version");
    print!("{}", cmd.render_version());

    // Hidden options still parse:
    let r = Resize::try_parse_from(["resize", "a.png", "--timings"]).unwrap();
    println!("\nhidden --timings still works: {r:?}");
}
