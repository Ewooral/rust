//! Beyond the Book: `Cow` (borrow when you can, own when you must), and the
//! string types for file paths and the operating system.
//! Run: cargo run -p collections --example cow_and_paths

use std::borrow::Cow;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Replaces tabs with spaces. Most input has no tabs, so it would be
/// wasteful to allocate a new String every time. `Cow` lets us return the
/// caller's text untouched (Borrowed) and only allocate when we change it
/// (Owned). The `'_` ties the borrowed case to `input` (lifetimes: lesson 03/01).
fn expand_tabs(input: &str) -> Cow<'_, str> {
    if input.contains('\t') {
        Cow::Owned(input.replace('\t', "    "))
    } else {
        Cow::Borrowed(input)
    }
}

fn main() {
    for line in ["no tabs here", "one\ttab"] {
        let out = expand_tabs(line);
        let kind = match &out {
            Cow::Borrowed(_) => "borrowed, no allocation",
            Cow::Owned(_) => "owned, newly allocated",
        };
        // A Cow<str> derefs to &str, so it is used like any string.
        println!("{out:?}: {kind}, {} bytes", out.len());
    }

    // Path / PathBuf are to file paths what &str / String are to text.
    // They handle separators for you (`\` on Windows, `/` on Linux).
    let mut path = PathBuf::from("lessons");
    path.push("collections");
    path.push("README.md");
    println!("path: {}", path.display());
    println!("file name: {:?}", path.file_name());
    println!("extension: {:?}", path.extension());
    println!("parent: {:?}", path.parent());

    let p: &Path = Path::new("Cargo.toml");
    println!("{} exists here? {}", p.display(), p.exists());

    // OsString / OsStr: text exactly as the operating system gives it, which
    // may not be valid UTF-8. Convert to a Rust string only when you need to.
    let os: OsString = OsString::from("report.txt");
    match os.to_str() {
        Some(s) => println!("valid UTF-8: {s}"),
        None => println!("not UTF-8: {}", os.to_string_lossy()),
    }
}
