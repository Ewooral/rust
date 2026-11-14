//! Beyond the Book: working with the file system and running other programs.
//! Run: cargo run -p minigrep --example fs_and_process

use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

fn main() -> io::Result<()> {
    // ---- std::fs ----
    let root = std::env::temp_dir().join("minigrep_fs_demo");
    let _ = fs::remove_dir_all(&root); // start clean; ignore "not found"
    fs::create_dir_all(root.join("notes/old"))?; // like `mkdir -p`
    fs::write(root.join("notes/today.txt"), "buy milk\ncall Ama\n")?;
    fs::write(root.join("notes/old/last_week.txt"), "pay rent\n")?;
    fs::copy(root.join("notes/today.txt"), root.join("notes/backup.txt"))?;

    println!("files under {}:", root.display());
    list_files(&root, 0)?;

    let meta = fs::metadata(root.join("notes/today.txt"))?;
    println!(
        "today.txt is {} bytes, is_file = {}",
        meta.len(),
        meta.is_file()
    );

    fs::rename(root.join("notes/backup.txt"), root.join("notes/copy.txt"))?;
    println!(
        "exists after rename? {}",
        root.join("notes/copy.txt").exists()
    );
    fs::remove_dir_all(&root)?;

    // ---- std::process::Command ----
    // Start another program, wait for it, and collect its output.
    // We run `cargo --version`: cargo is certainly installed if you can run this.
    let output = Command::new("cargo").arg("--version").output()?;
    println!(
        "cargo said: {} (exit status: {})",
        String::from_utf8_lossy(&output.stdout).trim(),
        output.status
    );

    // A failing program does not make `output()` fail: check the status.
    let bad = Command::new("cargo").arg("no-such-subcommand").output()?;
    println!(
        "bad command succeeded? {}, stderr starts: {:?}",
        bad.status.success(),
        String::from_utf8_lossy(&bad.stderr)
            .lines()
            .next()
            .unwrap_or("")
    );
    // A program that does not exist DOES fail, with io::ErrorKind::NotFound.
    match Command::new("definitely-not-a-real-program").output() {
        Ok(_) => println!("unexpected!"),
        Err(e) => println!("could not start it: {:?}", e.kind()),
    }
    Ok(())
}

/// Recursively prints a directory tree.
fn list_files(dir: &Path, depth: usize) -> io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.file_name()); // read_dir order is not guaranteed
    for entry in entries {
        let path = entry.path();
        println!(
            "{}{}",
            "  ".repeat(depth + 1),
            entry.file_name().to_string_lossy()
        );
        if path.is_dir() {
            list_files(&path, depth + 1)?;
        }
    }
    Ok(())
}
