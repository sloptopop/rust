//! Unit tests for [`super`] (Unix only: they set execute bits).

use std::fs;
use std::os::unix::fs::PermissionsExt;

use super::*;

/// A fresh, empty directory for one test, removed when dropped.
struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let dir = env::temp_dir().join(format!(
            "sloptopop-rust-locate-{}-{name}",
            std::process::id()
        ));
        drop(fs::remove_dir_all(&dir));
        fs::create_dir_all(&dir).expect("temp dir");
        Self(dir)
    }
}

impl std::ops::Deref for TempDir {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        drop(fs::remove_dir_all(&self.0));
    }
}

/// Creates `dir/sloptopop` with the given permission bits.
fn binary_in(dir: &Path, mode: u32) -> PathBuf {
    let path = dir.join("sloptopop");
    fs::write(&path, "#!/bin/sh\n").expect("write binary");
    fs::set_permissions(&path, fs::Permissions::from_mode(mode)).expect("chmod");
    path
}

fn joined(dirs: &[&Path]) -> Option<OsString> {
    env::join_paths(dirs).ok()
}

#[test]
fn explicit_path_wins_even_if_missing() {
    let dir = TempDir::new("explicit");
    let on_path = binary_in(&dir, 0o755);
    assert_eq!(
        search(
            Some(PathBuf::from("/explicit/sloptopop")),
            Some("/env/sloptopop".into()),
            Some(&dir.join("app")),
            joined(&[&dir]),
        ),
        Some(PathBuf::from("/explicit/sloptopop")),
        "explicit first"
    );
    assert!(on_path.exists(), "the others were available");
}

#[test]
fn env_var_comes_second_and_empty_means_unset() {
    let dir = TempDir::new("env");
    let bundled = binary_in(&dir, 0o755);
    assert_eq!(
        search(
            None,
            Some("/env/sloptopop".into()),
            Some(&dir.join("app")),
            None
        ),
        Some(PathBuf::from("/env/sloptopop")),
        "env var before bundled"
    );
    assert_eq!(
        search(None, Some(OsString::new()), Some(&dir.join("app")), None),
        Some(bundled),
        "empty env var ignored"
    );
}

#[test]
fn bundled_binary_comes_before_path() {
    let bundled_dir = TempDir::new("bundled");
    let path_dir = TempDir::new("bundled-path");
    let bundled = binary_in(&bundled_dir, 0o755);
    binary_in(&path_dir, 0o755);
    assert_eq!(
        search(
            None,
            None,
            Some(&bundled_dir.join("app")),
            joined(&[&path_dir])
        ),
        Some(bundled),
        "next to the executable"
    );
}

#[test]
fn path_is_searched_in_order_skipping_non_executables() {
    let empty = TempDir::new("path-empty");
    let not_executable = TempDir::new("path-noexec");
    let directory = TempDir::new("path-dir");
    let good = TempDir::new("path-good");
    let later = TempDir::new("path-later");
    binary_in(&not_executable, 0o644);
    fs::create_dir_all(directory.join("sloptopop")).expect("dir named sloptopop");
    let expected = binary_in(&good, 0o700);
    binary_in(&later, 0o755);
    assert_eq!(
        search(
            None,
            None,
            Some(&empty.join("app")),
            joined(&[&empty, &not_executable, &directory, &good, &later]),
        ),
        Some(expected),
        "first executable file on PATH"
    );
}

#[test]
fn only_execute_bits_count() {
    for (mode, executable) in [(0o644, false), (0o100, true), (0o010, true), (0o001, true)] {
        let dir = TempDir::new(&format!("mode-{mode:o}"));
        let path = binary_in(&dir, mode);
        assert_eq!(is_executable(&path), executable, "mode {mode:o}");
    }
}

#[test]
fn nothing_found() {
    let empty = TempDir::new("nothing");
    assert_eq!(
        search(None, None, Some(&empty.join("app")), joined(&[&empty])),
        None,
        "empty dirs"
    );
    assert_eq!(search(None, None, None, None), None, "no hints at all");
    assert_eq!(
        search(None, None, Some(Path::new("/")), None),
        None,
        "executable without parent"
    );
}

#[test]
fn binary_name_is_sloptopop() {
    assert_eq!(binary_name(), PathBuf::from("sloptopop"), "unix name");
}
