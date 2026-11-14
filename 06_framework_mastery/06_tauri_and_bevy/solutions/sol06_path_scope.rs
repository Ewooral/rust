//! Solution to exercise 6: a file-system scope check.
//!
//! Suppose the notes app gets a command `read_attachment(path)` that reads a
//! file from the app's attachments folder. The path comes from the web
//! page, so a bug or an injected script could ask for `../../.ssh/id_rsa`.
//! Tauri's file-system plugin has "scopes" for exactly this: only paths
//! inside allowed folders. Write the check:
//!
//! `resolve_in_scope(base, requested)` returns `base` joined with the
//! requested RELATIVE path, after cleaning it up, or an error:
//! - empty: `ScopeError::Empty`
//! - absolute or rooted (`/etc/passwd`, `C:\Windows`, `\\server\share`):
//!   `ScopeError::Absolute`
//! - `..` that climbs above `base` (`../secret`, `a/../../x`):
//!   `ScopeError::Escapes`
//! - `.` parts are dropped; `a/../b` is fine and means `b`.
//!
//! Work on `Path::new(requested).components()`: `Normal`, `CurDir`,
//! `ParentDir`, `RootDir` and `Prefix` (a Windows drive or share). Never
//! check paths with string searches such as `contains("..")`: they miss
//! cases and reject good names like `notes..txt`.
//!
//! (Real code also has to think about symbolic links, which can point
//! outside the folder: `fs::canonicalize` both and compare.)
//!
//! Test: cargo test -p tauri_and_bevy --features solutions --bin sol06_path_scope

use std::path::{Component, Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
enum ScopeError {
    Empty,
    Absolute,
    Escapes,
}

fn resolve_in_scope(base: &Path, requested: &str) -> Result<PathBuf, ScopeError> {
    if requested.trim().is_empty() {
        return Err(ScopeError::Empty);
    }
    let mut inside = PathBuf::new();
    let mut depth = 0usize;
    for component in Path::new(requested).components() {
        match component {
            Component::Normal(part) => {
                inside.push(part);
                depth += 1;
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if depth == 0 {
                    return Err(ScopeError::Escapes);
                }
                inside.pop();
                depth -= 1;
            }
            Component::RootDir | Component::Prefix(_) => return Err(ScopeError::Absolute),
        }
    }
    Ok(base.join(inside))
}

fn main() {
    let base = Path::new("attachments");
    for requested in [
        "receipt.pdf",
        "2026/may/./photo.png",
        "2026/../notes..txt",
        "../secrets.txt",
        "/etc/passwd",
        "",
    ] {
        println!("{requested:<24} -> {:?}", resolve_in_scope(base, requested));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> PathBuf {
        PathBuf::from("app").join("attachments")
    }

    #[test]
    fn plain_relative_paths_stay_inside() {
        assert_eq!(
            resolve_in_scope(&base(), "receipt.pdf"),
            Ok(base().join("receipt.pdf"))
        );
        assert_eq!(
            resolve_in_scope(&base(), "2026/may/photo.png"),
            Ok(base().join("2026").join("may").join("photo.png"))
        );
    }

    #[test]
    fn dots_are_cleaned_up() {
        assert_eq!(
            resolve_in_scope(&base(), "./a/./b.txt"),
            Ok(base().join("a").join("b.txt"))
        );
        assert_eq!(
            resolve_in_scope(&base(), "a/../b.txt"),
            Ok(base().join("b.txt"))
        );
        assert_eq!(
            resolve_in_scope(&base(), "notes..txt"),
            Ok(base().join("notes..txt")),
            "two dots inside a name are fine"
        );
    }

    #[test]
    fn climbing_out_is_refused() {
        assert_eq!(resolve_in_scope(&base(), ".."), Err(ScopeError::Escapes));
        assert_eq!(
            resolve_in_scope(&base(), "../secret.txt"),
            Err(ScopeError::Escapes)
        );
        assert_eq!(
            resolve_in_scope(&base(), "a/../../x"),
            Err(ScopeError::Escapes)
        );
    }

    #[test]
    fn absolute_paths_and_empty_input_are_refused() {
        assert_eq!(
            resolve_in_scope(&base(), "/etc/passwd"),
            Err(ScopeError::Absolute)
        );
        assert_eq!(resolve_in_scope(&base(), ""), Err(ScopeError::Empty));
        assert_eq!(resolve_in_scope(&base(), "   "), Err(ScopeError::Empty));
    }

    #[cfg(windows)]
    #[test]
    fn windows_drives_and_shares_are_refused() {
        assert_eq!(
            resolve_in_scope(&base(), r"C:\Windows\win.ini"),
            Err(ScopeError::Absolute)
        );
        assert_eq!(
            resolve_in_scope(&base(), r"\\server\share\file"),
            Err(ScopeError::Absolute)
        );
        assert_eq!(
            resolve_in_scope(&base(), r"a\..\..\x"),
            Err(ScopeError::Escapes)
        );
    }
}
