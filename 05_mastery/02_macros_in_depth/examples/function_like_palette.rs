//! A function-like procedural macro with its own syntax: `palette!` checks
//! colours at compile time and makes a module of constants.
//!
//! Run: cargo run -p macros_in_depth --example function_like_palette
//!
//! The macro is in `macros/src/palette.rs`. Its grammar is a `syn::parse::Parse`
//! impl: an optional visibility, `mod`, a name, then `{ name = "#hex", … }`.

use macros_in_depth::palette;

palette! {
    pub mod sunset {
        sky = "#ff7e5f",
        sun = "#feb47b",
        sea = "#2b5876",
        sand = "#fc0", // the short form: #fc0 means #ffcc00
    }
}

// A macro_rules! macro can't do this: it can match a string literal, but
// it can't look inside one. A procedural macro is ordinary Rust running in
// the compiler, so it parses the text and refuses bad colours:
//
//     palette! { mod bad { sky = "orange", sea = "#12" } }
//     error: expected a colour like "#ff8800" or "#f80"    (under "orange")
//     error: expected a colour like "#ff8800" or "#f80"    (under "#12")
//     Both are reported in one build: the macro combines its errors.
//
//     palette! { mod bad { sky: "#fff" } }
//     error: expected `=`              (syn's own parse error, under the `:`)
//
//     palette! { mod bad { sky = "#fff", sky = "#000" } }
//     error: `sky` is in this palette twice

fn main() {
    println!("sunset::SKY = {}", sunset::SKY);
    println!("sunset::SAND = {:?}", sunset::SAND);
    for (name, colour) in sunset::ALL {
        let swatch = colour.paint("████");
        println!("{swatch} {name:<5} {colour}");
    }
    // The constants are computed by the macro, at compile time:
    const SEA_BLUE: u8 = sunset::SEA.b;
    println!("the sea's blue channel, as a const: {SEA_BLUE}");
}
