//! Adam's per-user config and cache directories.
//!
//! Earlier versions used other directory names (`apple-music-linux`, `aml`). Their contents are
//! folded into the current directory on first use.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The directory name under the OS config and cache roots.
pub const NAME: &str = "adam";

const LEGACY_NAMES: [&str; 2] = ["apple-music-linux", "aml"];

#[derive(Clone, Copy)]
enum Kind {
    Config,
    Cache,
}

/// The per-user config directory, for example `~/.config/adam`.
pub fn config_dir() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| resolve(Kind::Config))
}

/// The per-user cache directory, for example `~/.cache/adam`.
pub fn cache_dir() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| resolve(Kind::Cache))
}

fn root(kind: Kind) -> PathBuf {
    let os_dir = match kind {
        Kind::Config => dirs::config_dir(),
        Kind::Cache => dirs::cache_dir(),
    };
    os_dir.unwrap_or_else(|| match (dirs::home_dir(), kind) {
        (Some(home), Kind::Config) => home.join(".config"),
        (Some(home), Kind::Cache) => home.join(".cache"),
        (None, _) => std::env::temp_dir(),
    })
}

fn resolve(kind: Kind) -> PathBuf {
    let root = root(kind);
    let dir = root.join(NAME);
    for old in LEGACY_NAMES {
        merge(&root.join(old), &dir);
    }
    dir
}

fn exists(path: &Path) -> bool {
    !matches!(fs::symlink_metadata(path), Err(e) if e.kind() == ErrorKind::NotFound)
}

/// Moves everything in `src` into `dst` without overwriting. When `dst` does not exist this is
/// one directory rename. Otherwise it goes entry by entry and recurses into directories that
/// exist on both sides. A file that exists on both sides keeps the `dst` copy, and the `src`
/// copy stays where it is.
fn merge(src: &Path, dst: &Path) {
    if !fs::symlink_metadata(src).is_ok_and(|m| m.is_dir()) {
        return;
    }
    if !exists(dst) {
        let parent_ready = dst.parent().is_none_or(|p| create_private_dir(p).is_ok());
        if parent_ready && fs::rename(src, dst).is_ok() {
            return;
        }
    }
    if create_private_dir(dst).is_err() {
        return;
    }
    let Ok(entries) = fs::read_dir(src) else {
        return;
    };
    for entry in entries.flatten() {
        let (from, to) = (entry.path(), dst.join(entry.file_name()));
        if !exists(&to) {
            let _ = fs::rename(&from, &to);
        } else if entry.file_type().is_ok_and(|t| t.is_dir()) {
            merge(&from, &to);
        }
    }
    // Succeeds only when nothing was left behind.
    let _ = fs::remove_dir(src);
}

/// Renames `old_path` to `new_path` when only the old one exists. For one-off relocations of
/// paths that do not live under the legacy directory names.
pub fn move_into(old_path: &Path, new_path: &Path) {
    if fs::symlink_metadata(new_path).is_ok() || fs::symlink_metadata(old_path).is_err() {
        return;
    }
    if new_path
        .parent()
        .is_none_or(|p| create_private_dir(p).is_ok())
    {
        let _ = fs::rename(old_path, new_path);
    }
}

/// `mkdir -p` with mode 0700 on Unix.
fn create_private_dir(path: &Path) -> std::io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_renames_when_destination_is_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let (src, dst) = (tmp.path().join("aml"), tmp.path().join("adam"));
        fs::create_dir(&src).unwrap();
        fs::write(src.join("a"), "old").unwrap();

        merge(&src, &dst);

        assert_eq!(fs::read_to_string(dst.join("a")).unwrap(), "old");
        assert!(!src.exists());
    }

    #[test]
    fn merge_keeps_the_destination_copy_on_conflict() {
        let tmp = tempfile::tempdir().unwrap();
        let (src, dst) = (tmp.path().join("aml"), tmp.path().join("adam"));
        fs::create_dir_all(src.join("sub")).unwrap();
        fs::create_dir_all(dst.join("sub")).unwrap();
        fs::write(src.join("both"), "old").unwrap();
        fs::write(dst.join("both"), "new").unwrap();
        fs::write(src.join("sub/only-old"), "old").unwrap();

        merge(&src, &dst);

        assert_eq!(fs::read_to_string(dst.join("both")).unwrap(), "new");
        assert_eq!(fs::read_to_string(dst.join("sub/only-old")).unwrap(), "old");
        // The conflicting source file is left in place, so the source directory stays.
        assert_eq!(fs::read_to_string(src.join("both")).unwrap(), "old");
    }

    #[test]
    fn move_into_does_not_overwrite() {
        let tmp = tempfile::tempdir().unwrap();
        let (old, new) = (tmp.path().join("old"), tmp.path().join("nested/new"));
        fs::write(&old, "old").unwrap();

        move_into(&old, &new);
        assert_eq!(fs::read_to_string(&new).unwrap(), "old");

        fs::write(&old, "second").unwrap();
        move_into(&old, &new);
        assert_eq!(fs::read_to_string(&new).unwrap(), "old");
    }
}
