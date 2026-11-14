//! A derive macro: `#[derive(Settings)]` reads a struct's fields, types,
//! doc comments and `#[setting(…)]` helper attributes, and writes an impl.
//!
//! Run: cargo run -p macros_in_depth --example derive_settings
//!
//! The macro is in `macros/src/settings.rs`; the trait it implements is in
//! `src/settings.rs`.

use macros_in_depth::Settings;

/// Colours and sizes for a text editor.
#[derive(Settings)]
struct EditorSettings {
    /// Spaces per tab.
    #[setting(default = "4")]
    tab_width: u8,
    /// Wrap long lines.
    #[setting(rename = "word-wrap", default = "true")]
    word_wrap: bool,
    /// Colour theme name.
    theme: String,
    /// Font size in points; the system default if not set.
    #[setting(rename = "font-size")]
    font_size: Option<f32>,
}

// What the derive writes (simplified; `cargo expand` shows the real thing):
//
//     impl ::macros_in_depth::Settings for EditorSettings {
//         fn from_map(map: &BTreeMap<String, String>) -> Result<Self, SettingsError> {
//             Ok(Self {
//                 tab_width: __private::required(map, "tab_width", Some("4"))?,
//                 word_wrap: __private::required(map, "word-wrap", Some("true"))?,
//                 theme:     __private::required(map, "theme", None)?,
//                 font_size: __private::optional(map, "font-size", None)?,
//             })
//         }
//         fn fields() -> &'static [FieldInfo] { /* key, type, default, doc */ }
//     }
//
// Mistakes the derive catches, each pointing at the exact spot:
//
//     #[derive(Settings)] enum Mode { Fast }
//     error: #[derive(Settings)] only works on structs: an enum has no fields to fill in
//
//     #[derive(Settings)] struct Pair(u8, u8);
//     error: #[derive(Settings)] needs named fields, like `struct Config { port: u16 }`
//
//     #[setting(defualt = "4")]          (a typo, underlined)
//     error: unknown setting option: expected `default` or `rename`
//
//     #[setting(default = 4)]            (the 4 is underlined)
//     error: write the default as it would appear in the settings file, in quotes: `default = "8080"`
//
//     colour: Colour,                    (a type without FromStr)
//     error[E0277]: the trait bound `Colour: FromStr` is not satisfied
//     The error points at `Colour` in the struct, because the generated code
//     for that field carries the field type's span (`quote_spanned!`).

fn main() {
    println!("help, from the doc comments:\n{}", EditorSettings::help());

    let text = "theme = solarized\nword-wrap = false\nfont-size = 13.5\n";
    let settings = EditorSettings::from_text(text).expect("valid settings");
    println!(
        "loaded: theme {:?}, tab width {}, word wrap {}, font size {:?}\n",
        settings.theme, settings.tab_width, settings.word_wrap, settings.font_size
    );

    for bad in [
        "tab_width = 4",              // `theme` is required
        "theme = x\ntab_width = 300", // 300 doesn't fit in a u8
        "theme = x\nword_wrap = no",  // the key is `word-wrap`
    ] {
        if let Err(error) = EditorSettings::from_text(bad) {
            println!("{bad:?}\n  -> {error}");
        }
    }
}
