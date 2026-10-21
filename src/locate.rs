//! Finding the `sloptopop` binary, in the order `PROTOCOL.md` prescribes.

use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Environment variable naming the binary to run.
pub(crate) const ENV_VAR: &str = "SLOPTOPOP_BIN";

/// Looks up the binary for this process: `explicit`, then
/// [`ENV_VAR`], then next to the current executable, then `PATH`.
/// `None` if none of them yields one.
pub(crate) fn find(explicit: Option<PathBuf>) -> Option<PathBuf> {
    search(
        explicit,
        env::var_os(ENV_VAR),
        env::current_exe().ok().as_deref(),
        env::var_os("PATH"),
    )
}

/// First match wins: an explicit path and a non-empty `env_bin` are used as
/// given; the binary next to `exe` and on `path` must be an executable file.
fn search(
    explicit: Option<PathBuf>,
    env_bin: Option<OsString>,
    exe: Option<&Path>,
    path: Option<OsString>,
) -> Option<PathBuf> {
    explicit
        .or_else(|| env_bin.filter(|v| !v.is_empty()).map(PathBuf::from))
        .or_else(|| {
            exe.and_then(Path::parent)
                .map(|dir| dir.join(binary_name()))
                .filter(|candidate| is_executable(candidate))
        })
        .or_else(|| {
            env::split_paths(&path?)
                .map(|dir| dir.join(binary_name()))
                .find(|candidate| is_executable(candidate))
        })
}

/// `sloptopop`, plus `.exe` on Windows.
fn binary_name() -> PathBuf {
    Path::new("sloptopop").with_extension(env::consts::EXE_EXTENSION)
}

/// Whether `path` is a file this process may execute: on Unix, a file
/// with at least one execute bit; elsewhere, any file.
fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = path.metadata() else {
        return false;
    };
    #[cfg(unix)]
    let executable = {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    };
    #[cfg(not(unix))]
    let executable = true;
    metadata.is_file() && executable
}

#[cfg(test)]
#[cfg(unix)]
mod tests;
