//! Finding `Cargo.toml` files and workspace members on disk (Book 14.3).

use std::fs;
use std::path::{Path, PathBuf};

use crate::manifest::Manifest;

/// The nearest `Cargo.toml`: `start` itself if it is a file, otherwise the
/// first `Cargo.toml` found in `start` or one of its parent folders. This is
/// how Cargo itself finds the manifest when you run `cargo build`.
pub fn find_manifest(start: &Path) -> Option<PathBuf> {
    if start.is_file() {
        return Some(start.to_path_buf());
    }
    start
        .ancestors()
        .map(|dir| dir.join("Cargo.toml"))
        .find(|candidate| candidate.is_file())
}

/// The workspace that `manifest_path` belongs to: the nearest `Cargo.toml`
/// with a `[workspace]` table, looking in the manifest's own folder first,
/// then upwards. Returns its path and its parsed contents.
pub fn find_workspace_root(manifest_path: &Path) -> Option<(PathBuf, Manifest)> {
    let folder = manifest_path.parent()?;
    folder.ancestors().find_map(|dir| {
        let candidate = dir.join("Cargo.toml");
        let text = fs::read_to_string(&candidate).ok()?;
        let manifest = Manifest::parse(&text).ok()?;
        manifest.is_workspace().then_some((candidate, manifest))
    })
}

/// The folder of every member of a workspace, in the order listed.
///
/// A member ending in `/*` (such as `"crates/*"`) means every sub-folder
/// that has a `Cargo.toml`.
pub fn member_dirs(root_dir: &Path, root: &Manifest) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for member in root.workspace_members() {
        if let Some(parent) = member.strip_suffix("/*") {
            let Ok(entries) = fs::read_dir(root_dir.join(parent)) else {
                continue;
            };
            let mut found: Vec<PathBuf> = entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.join("Cargo.toml").is_file())
                .collect();
            found.sort();
            dirs.extend(found);
        } else {
            dirs.push(root_dir.join(member));
        }
    }
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn this_crate() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn finds_this_lessons_own_manifest_from_a_sub_folder() {
        let found = find_manifest(&this_crate().join("src")).unwrap();
        assert_eq!(found, this_crate().join("Cargo.toml"));
    }

    #[test]
    fn this_lesson_belongs_to_the_course_workspace() {
        let (root_path, root) = find_workspace_root(&this_crate().join("Cargo.toml")).unwrap();
        assert!(root_path.ends_with("Cargo.toml"));
        let members = member_dirs(root_path.parent().unwrap(), &root);
        assert!(
            members
                .iter()
                .any(|dir| dir.ends_with("05_cargo_and_crates_io"))
        );
    }
}
