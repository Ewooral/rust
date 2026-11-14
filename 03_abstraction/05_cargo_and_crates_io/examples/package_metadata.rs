//! Book 14.2: the [package] metadata that crates.io shows on a crate's page.
//!
//! Run: cargo run -p cargo_and_crates_io --example package_metadata
//!
//! Cargo hands the [package] fields to the compiler as CARGO_PKG_*
//! environment variables, and `env!` reads them AT COMPILE TIME. That is how
//! programs print their own version with no copy to keep in sync.

use cargo_and_crates_io::{Manifest, Value};

fn main() -> Result<(), cargo_and_crates_io::ParseError> {
    println!("name:          {}", env!("CARGO_PKG_NAME"));
    // version comes from [workspace.package] in the root Cargo.toml.
    println!("version:       {}", env!("CARGO_PKG_VERSION"));
    println!(
        "  as numbers:  major {} minor {} patch {}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH")
    );
    println!("description:   {}", env!("CARGO_PKG_DESCRIPTION"));
    println!("license:       {}", env!("CARGO_PKG_LICENSE"));
    println!("rust-version:  {}", env!("CARGO_PKG_RUST_VERSION"));
    println!("readme:        {}", env!("CARGO_PKG_README"));
    // Fields that are not set are empty strings, not errors:
    println!("repository:    {:?}", env!("CARGO_PKG_REPOSITORY"));
    println!("authors:       {:?}", env!("CARGO_PKG_AUTHORS"));

    // keywords, categories and publish have no CARGO_PKG_ variable, so read
    // Cargo.toml itself (include_str! embeds the file in the program).
    let manifest = Manifest::parse(include_str!("../Cargo.toml"))?;
    for key in ["keywords", "categories", "publish"] {
        let shown = match manifest.get("package", key) {
            Some(Value::Array(items)) => {
                let words: Vec<&str> = items.iter().filter_map(Value::as_str).collect();
                format!("{words:?}")
            }
            Some(Value::Boolean(b)) => b.to_string(),
            other => format!("{other:?}"),
        };
        println!("{:<14} {shown}", format!("{key}:"));
    }

    // The classic use: a --version line that never goes out of date.
    println!("\n{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    Ok(())
}
