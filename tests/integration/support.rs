//! Helpers shared by the integration tests.

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use sloptopop::Options;

/// The CLI repository: `../cli` next to this crate. cargo-mutants builds a
/// copy of this crate elsewhere, so `$PWD` (still the original directory
/// there) is tried as well.
pub(crate) fn cli_dir() -> PathBuf {
    let candidates = [
        Some(PathBuf::from(env!("CARGO_MANIFEST_DIR"))),
        env::var_os("PWD").map(PathBuf::from),
    ];
    candidates
        .into_iter()
        .flatten()
        .map(|dir| dir.join("../cli"))
        .find(|dir| dir.join("conformance").is_dir())
        .expect("the sloptopop CLI repository should be at ../cli")
}

/// The real binary: `SLOPTOPOP_BIN`, or the CLI repository's debug build.
pub(crate) fn binary() -> PathBuf {
    env::var_os("SLOPTOPOP_BIN")
        .filter(|v| !v.is_empty())
        .map_or_else(|| cli_dir().join("target/debug/sloptopop"), PathBuf::from)
}

/// Options that run the real binary.
pub(crate) fn options() -> Options {
    Options::default().binary(binary())
}

/// Whether process `pid` exists (a zombie counts: it was not waited for).
pub(crate) fn alive(pid: u32) -> bool {
    Command::new("sh")
        .args(["-c", &format!("kill -0 {pid}")])
        .stderr(Stdio::null())
        .status()
        .expect("sh should run")
        .success()
}

/// A fresh, empty directory for one test, removed when dropped.
#[derive(Debug)]
pub(crate) struct TempDir(PathBuf);

impl TempDir {
    pub(crate) fn new(name: &str) -> Self {
        let dir = env::temp_dir().join(format!("sloptopop-rust-it-{}-{name}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).expect("remove old temp dir");
        }
        std::fs::create_dir_all(&dir).expect("create temp dir");
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
        drop(std::fs::remove_dir_all(&self.0));
    }
}

/// Writes an executable shell script. A child `sh` writes it, so this
/// (multi-threaded) process never holds a writable descriptor to a file it
/// may execute, which could make `exec` fail with "text file busy".
pub(crate) fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    let status = Command::new("sh")
        .args(["-c", r#"printf '%s' "$1" > "$2" && chmod 755 "$2""#, "sh"])
        .arg(format!("#!/bin/sh\n{body}\n"))
        .arg(&path)
        .status()
        .expect("sh should run");
    assert!(status.success(), "writing {}", path.display());
    path
}
