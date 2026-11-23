//! Platform helpers (`platform.mjs`): config root, engine library environment, migration from
//! the app's earlier names. Everything that depends on the OS takes a [`Platform`] and an
//! [`Env`] so it is testable from any OS.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

/// The operating system a function computes for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    Windows,
}

impl Platform {
    /// The platform this binary runs on.
    pub const fn current() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else {
            Self::Linux
        }
    }

    const fn path_sep(self) -> char {
        match self {
            Self::Linux => '/',
            Self::Windows => '\\',
        }
    }

    const fn list_sep(self) -> char {
        match self {
            Self::Linux => ':',
            Self::Windows => ';',
        }
    }

    /// Joins path parts with this platform's separator (the target's, not the host's).
    fn join(self, parts: &[&str]) -> String {
        let sep = self.path_sep();
        let mut out = String::new();
        for part in parts.iter().filter(|p| !p.is_empty()) {
            if !out.is_empty() && !out.ends_with(sep) {
                out.push(sep);
            }
            out.push_str(part);
        }
        out
    }
}

/// A snapshot of environment variables. Empty values count as unset, as `env.X || …` does in
/// the Electron code.
#[derive(Debug, Clone, Default)]
pub struct Env(HashMap<String, String>);

impl Env {
    /// The current process environment (variables that are not valid UTF-8 are skipped).
    pub fn from_process() -> Self {
        Self(std::env::vars().collect())
    }

    /// The value of `name`, or `None` when it is unset or empty.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0
            .get(name)
            .map(String::as_str)
            .filter(|v| !v.is_empty())
    }
}

impl<K: Into<String>, V: Into<String>> FromIterator<(K, V)> for Env {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }
}

/// Directory that holds the `adam` config folder. Windows: `%APPDATA%`. Inside a Flatpak the
/// real `~/.config` is invisible and `XDG_CONFIG_HOME` is the app's own directory.
pub fn config_root(platform: Platform, env: &Env, home: &Path) -> PathBuf {
    match platform {
        Platform::Windows => env.get("APPDATA").map(PathBuf::from).unwrap_or_else(|| {
            PathBuf::from(Platform::Windows.join(&[&home.to_string_lossy(), "AppData", "Roaming"]))
        }),
        Platform::Linux => match (env.get("FLATPAK_ID"), env.get("XDG_CONFIG_HOME")) {
            (Some(_), Some(xdg)) => PathBuf::from(xdg),
            _ => home.join(".config"),
        },
    }
}

/// Where the engine's bundled native libraries are, and which of them exist.
#[derive(Debug, Clone, Default)]
pub struct EngineLibs {
    pub vlc_dir: String,
    pub ffmpeg_dir: String,
    /// The engine executable's own folder (Windows DLL lookup).
    pub resources_dir: Option<String>,
    pub has_vlc: bool,
    pub has_ffmpeg: bool,
}

/// Environment additions that let the engine find its bundled native libraries. Linux uses
/// `LD_LIBRARY_PATH`; Windows resolves DLLs through `PATH` (and the exe's own folder).
pub fn engine_lib_env(platform: Platform, env: &Env, libs: &EngineLibs) -> Vec<(String, String)> {
    let sep = platform.list_sep().to_string();
    let join_list = |items: Vec<Option<String>>| {
        items
            .into_iter()
            .flatten()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(&sep)
    };
    let plugins = || platform.join(&[&libs.vlc_dir, "plugins"]);
    let mut out = Vec::new();
    match platform {
        Platform::Windows => {
            let mut dirs = Vec::new();
            if libs.has_vlc {
                dirs.push(Some(libs.vlc_dir.clone()));
            }
            if libs.has_ffmpeg {
                dirs.push(Some(platform.join(&[&libs.ffmpeg_dir, "bin"])));
            }
            dirs.push(libs.resources_dir.clone());
            if libs.has_vlc {
                out.push(("VLC_PLUGIN_PATH".into(), plugins()));
            }
            dirs.push(env.get("PATH").or(env.get("Path")).map(str::to_owned));
            out.push(("PATH".into(), join_list(dirs)));
        }
        Platform::Linux => {
            if libs.has_vlc {
                let ld = join_list(vec![
                    Some(libs.vlc_dir.clone()),
                    Some(platform.join(&[&libs.ffmpeg_dir, "lib"])),
                    env.get("LD_LIBRARY_PATH").map(str::to_owned),
                ]);
                out.push(("LD_LIBRARY_PATH".into(), ld));
                out.push(("VLC_PLUGIN_PATH".into(), plugins()));
            }
            if libs.has_ffmpeg {
                let path = join_list(vec![
                    Some(platform.join(&[&libs.ffmpeg_dir, "bin"])),
                    env.get("PATH").map(str::to_owned),
                ]);
                out.push(("PATH".into(), path));
            }
        }
    }
    out
}

/// A YAML double-quoted scalar. JSON string syntax is a subset of it, so backslashes in Windows
/// paths and any quotes are escaped correctly.
pub fn yaml_str(v: &str) -> String {
    Value::from(v).to_string()
}

// ── Migration from the app's earlier names ───────────────────────────────────

const LEGACY_DIR_NAMES: [&str; 2] = ["apple-music-linux", "aml"];

/// Moves `src` into `dst` without overwriting: renames the whole directory when `dst` is
/// absent, otherwise entry by entry (recursing into directories present on both sides).
fn merge_dir(src: &Path, dst: &Path) {
    if !src.is_dir() {
        return;
    }
    if !dst.exists() {
        let renamed = dst.parent().is_none_or(|p| fs::create_dir_all(p).is_ok())
            && fs::rename(src, dst).is_ok();
        if renamed {
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
        let (s, d) = (entry.path(), dst.join(entry.file_name()));
        if !d.exists() {
            let _ = fs::rename(&s, &d);
        } else if s.is_dir() && d.is_dir() {
            merge_dir(&s, &d);
        }
    }
    // Fails when something could not move: then the old directory stays.
    let _ = fs::remove_dir(src);
}

/// `mkdir -p` with mode 0700 for the directories it creates (Unix).
pub(crate) fn create_private_dir(dir: &Path) -> std::io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(dir)
}

/// Folds the config directories earlier versions used into `<root>/adam`, and gives the
/// `aml-*` files inside it their new `adam-*` names. Returns `<root>/adam`.
pub fn migrate_legacy_config(root: &Path) -> PathBuf {
    let dir = root.join("adam");
    for old in LEGACY_DIR_NAMES {
        merge_dir(&root.join(old), &dir);
    }
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(rest) = name.to_str().and_then(|n| n.strip_prefix("aml-")) else {
                continue;
            };
            let to = dir.join(format!("adam-{rest}"));
            if !to.exists() {
                let _ = fs::rename(entry.path(), to);
            }
        }
    }
    dir
}

/// Keys the encrypted store used before the rename: `aml_x` → `adam_x`, never clobbering an
/// existing new key. Returns whether anything changed.
pub fn migrate_store_keys(store: &mut Map<String, Value>) -> bool {
    let old: Vec<String> = store
        .keys()
        .filter(|k| k.starts_with("aml_"))
        .cloned()
        .collect();
    for key in &old {
        let value = store.remove(key).unwrap_or(Value::Null);
        let new = format!("adam_{}", &key[4..]);
        store.entry(new).or_insert(value);
    }
    !old.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn config_root_per_platform_and_flatpak() {
        let env = |pairs: &[(&str, &str)]| pairs.iter().copied().collect::<Env>();
        assert_eq!(
            config_root(
                Platform::Windows,
                &env(&[("APPDATA", r"C:\Users\a\AppData\Roaming")]),
                Path::new(r"C:\Users\a")
            ),
            PathBuf::from(r"C:\Users\a\AppData\Roaming")
        );
        assert!(
            config_root(Platform::Windows, &env(&[]), Path::new("/h"))
                .to_string_lossy()
                .ends_with(r"AppData\Roaming")
        );
        assert_eq!(
            config_root(Platform::Linux, &env(&[]), Path::new("/home/a")),
            PathBuf::from("/home/a/.config")
        );
        assert_eq!(
            config_root(
                Platform::Linux,
                &env(&[("FLATPAK_ID", "x"), ("XDG_CONFIG_HOME", "/flat")]),
                Path::new("/home/a")
            ),
            PathBuf::from("/flat")
        );
        assert_eq!(
            config_root(
                Platform::Linux,
                &env(&[("XDG_CONFIG_HOME", "/flat")]),
                Path::new("/home/a")
            ),
            PathBuf::from("/home/a/.config")
        );
    }

    #[test]
    fn engine_lib_env_linux_and_windows() {
        let get = |v: &[(String, String)], k: &str| {
            v.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone())
        };
        let env: Env = [("PATH", "/usr/bin"), ("LD_LIBRARY_PATH", "/old")]
            .into_iter()
            .collect();
        let libs = EngineLibs {
            vlc_dir: "/r/vlc".into(),
            ffmpeg_dir: "/r/ff".into(),
            resources_dir: Some("/r".into()),
            has_vlc: true,
            has_ffmpeg: true,
        };
        let lin = engine_lib_env(Platform::Linux, &env, &libs);
        assert_eq!(
            get(&lin, "LD_LIBRARY_PATH").unwrap(),
            "/r/vlc:/r/ff/lib:/old"
        );
        assert_eq!(get(&lin, "VLC_PLUGIN_PATH").unwrap(), "/r/vlc/plugins");
        assert_eq!(get(&lin, "PATH").unwrap(), "/r/ff/bin:/usr/bin");

        let env: Env = [("PATH", r"C:\Windows")].into_iter().collect();
        let libs = EngineLibs {
            vlc_dir: r"C:\r\vlc".into(),
            ffmpeg_dir: r"C:\r\ff".into(),
            resources_dir: Some(r"C:\r".into()),
            has_vlc: true,
            has_ffmpeg: true,
        };
        let win = engine_lib_env(Platform::Windows, &env, &libs);
        let path = get(&win, "PATH").unwrap();
        assert!(path.starts_with(r"C:\r\vlc;"));
        assert!(path.ends_with(r";C:\Windows"));
        assert!(path.contains(r";C:\r;"));
        assert_eq!(get(&win, "LD_LIBRARY_PATH"), None);

        assert!(
            engine_lib_env(Platform::Linux, &Env::default(), &EngineLibs::default()).is_empty()
        );
    }

    #[test]
    fn yaml_str_escapes_windows_paths() {
        assert_eq!(
            yaml_str(r"C:\Users\a\AppData"),
            r#""C:\\Users\\a\\AppData""#
        );
        assert_eq!(yaml_str("/home/a/.config/x"), r#""/home/a/.config/x""#);
        assert_eq!(yaml_str("a\"b"), r#""a\"b""#);
    }

    fn put(p: &Path, c: &str) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, c).unwrap();
    }
    fn read(p: &Path) -> String {
        fs::read_to_string(p).unwrap()
    }

    #[test]
    fn migrate_legacy_config_moves_old_folder_and_renames_aml_files() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path();
        put(
            &root.join("apple-music-linux/electron-prefs.json"),
            r#"{"a":1}"#,
        );
        put(&root.join("apple-music-linux/aml-store.enc"), "enc");
        put(&root.join("apple-music-linux/drm/files/MUSIC_TOKEN"), "tok");
        put(&root.join("aml/eq.json"), "eq");
        let dir = migrate_legacy_config(root);
        assert_eq!(dir, root.join("adam"));
        assert_eq!(read(&dir.join("electron-prefs.json")), r#"{"a":1}"#);
        assert_eq!(read(&dir.join("adam-store.enc")), "enc");
        assert_eq!(read(&dir.join("drm/files/MUSIC_TOKEN")), "tok");
        assert_eq!(read(&dir.join("eq.json")), "eq");
        assert!(!root.join("apple-music-linux").exists());
        assert!(!root.join("aml").exists());
    }

    #[test]
    fn migrate_legacy_config_never_overwrites_and_is_idempotent() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path();
        put(&root.join("adam/electron-prefs.json"), "new");
        put(&root.join("adam/adam-store.enc"), "newstore");
        put(&root.join("apple-music-linux/electron-prefs.json"), "old");
        put(&root.join("apple-music-linux/extra.txt"), "x");
        put(&root.join("adam/aml-store.enc"), "stale");
        migrate_legacy_config(root);
        migrate_legacy_config(root);
        assert_eq!(read(&root.join("adam/electron-prefs.json")), "new");
        assert_eq!(read(&root.join("adam/adam-store.enc")), "newstore");
        assert_eq!(read(&root.join("adam/extra.txt")), "x");
        assert_eq!(
            read(&root.join("apple-music-linux/electron-prefs.json")),
            "old"
        );
    }

    #[test]
    fn migrate_legacy_config_on_fresh_machine() {
        let t = tempfile::tempdir().unwrap();
        assert_eq!(migrate_legacy_config(t.path()), t.path().join("adam"));
    }

    #[test]
    fn migrate_store_keys_renames_without_clobbering() {
        let mut st = json!({ "aml_resume_state": { "t": 1 }, "aml_play_counts": { "a": 1 }, "adam_play_counts": { "b": 2 }, "other": 3 })
            .as_object()
            .unwrap()
            .clone();
        assert!(migrate_store_keys(&mut st));
        assert_eq!(
            Value::Object(st.clone()),
            json!({ "adam_resume_state": { "t": 1 }, "adam_play_counts": { "b": 2 }, "other": 3 })
        );
        assert!(!migrate_store_keys(&mut st));
    }
}
