//! How to start a client: the caller's choices, before any process exists.

use std::path::PathBuf;

/// How to start a [`Client`](crate::client::Client).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Options {
    /// The binary to run. If `None`, it is looked up: the `SLOPTOPOP_BIN`
    /// environment variable, then `sloptopop` next to the current
    /// executable, then `sloptopop` on `PATH`.
    pub binary: Option<PathBuf>,
}

impl Options {
    /// Runs the binary at `path` instead of looking one up.
    #[must_use]
    pub fn binary(mut self, path: impl Into<PathBuf>) -> Self {
        self.binary = Some(path.into());
        self
    }
}

#[cfg(test)]
mod tests;
