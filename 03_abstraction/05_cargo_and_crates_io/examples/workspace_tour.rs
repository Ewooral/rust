//! Book 14.3: Cargo workspaces. This course IS one, so take a tour of it.
//!
//! Run: cargo run -p cargo_and_crates_io --example workspace_tour
//!
//! A workspace is a set of packages that share ONE Cargo.lock and ONE
//! target/ folder. The root Cargo.toml lists the members.

use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::path::Path;

use cargo_and_crates_io::Value;
use cargo_and_crates_io::workspace::{find_workspace_root, member_dirs};

fn main() -> Result<(), Box<dyn Error>> {
    let this_crate = Path::new(env!("CARGO_MANIFEST_DIR"));
    let (root_path, root) =
        find_workspace_root(&this_crate.join("Cargo.toml")).ok_or("no workspace found")?;
    let root_dir = root_path.parent().ok_or("the root has no folder")?;
    println!("workspace root:  {}", root_path.display());

    // [workspace] resolver = "3" picks how features and versions are resolved.
    if let Some(Value::String(resolver)) = root.get("workspace", "resolver") {
        println!("resolver:        {resolver}");
    }

    // [workspace.package]: values members can inherit with `x.workspace = true`.
    for key in ["version", "edition"] {
        if let Some(Value::String(v)) = root.get("workspace.package", key) {
            println!("shared {key:<9} {v}   (members say `{key}.workspace = true`)");
        }
    }

    // Members, grouped by their top-level folder.
    let members = member_dirs(root_dir, &root);
    let mut groups: BTreeMap<String, usize> = BTreeMap::new();
    for dir in &members {
        let relative = dir.strip_prefix(root_dir).unwrap_or(dir);
        let group = relative.components().next().map_or_else(String::new, |c| {
            c.as_os_str().to_string_lossy().into_owned()
        });
        *groups.entry(group).or_default() += 1;
    }
    println!("\n{} members:", members.len());
    for (group, count) in &groups {
        println!("  {group:<28} {count}");
    }

    // ONE lock file for everyone: every member gets the same versions.
    println!(
        "\nshared Cargo.lock:          {}",
        root_dir.join("Cargo.lock").is_file()
    );
    println!(
        "Cargo.lock in this lesson:  {}",
        this_crate.join("Cargo.lock").is_file()
    );

    // ONE target folder: this program was built into it, not into the
    // lesson's folder (unless CARGO_TARGET_DIR points somewhere else).
    println!(
        "this program was built at:  {}",
        env::current_exe()?.display()
    );
    Ok(())
}

// To make one member use another, add a PATH dependency in its Cargo.toml:
//
//     [dependencies]
//     add_one = { path = "../add_one" }
//
// Then, from the root:  cargo run -p adder    cargo test -p add_one
