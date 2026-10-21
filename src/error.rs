//! The error type of every fallible client operation.

use std::fmt;
use std::io;
use std::path::PathBuf;

use serde_json::Value;

use crate::framing::FrameError;

/// Everything that can go wrong talking to `sloptopop`.
///
/// [`Error::Rpc`] is the server answering a request with an error; the
/// session is still usable. Every other variant is a transport problem
/// ([`Error::is_transport`]): the client kills the process and further
/// requests fail.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The server answered with a JSON-RPC error object.
    Rpc {
        /// One of the codes in [`code`](crate::code).
        code: i64,
        /// Human-readable description.
        message: String,
        /// Structured detail, if any (e.g. `{"supported": [1]}`).
        data: Option<Value>,
    },
    /// No binary was found: no explicit path, no `SLOPTOPOP_BIN`, no
    /// `sloptopop` next to the current executable or on `PATH`.
    BinaryNotFound,
    /// The chosen binary could not be started.
    Spawn {
        /// The binary that was tried.
        path: PathBuf,
        /// Why it could not be started.
        source: io::Error,
    },
    /// The process closed its stdout (it exited or crashed) before replying.
    ProcessDied,
    /// Reading from or writing to the process failed, or its output ended
    /// in the middle of a frame.
    Io(io::Error),
    /// The process wrote something that is not a valid frame.
    Framing(FrameError),
    /// A frame that is not a valid JSON-RPC reply, or a reply whose result
    /// does not have the documented shape.
    InvalidReply(String),
    /// A reply to a request this client did not send next.
    UnexpectedId {
        /// The id of the request waiting for its reply.
        expected: u64,
        /// The id the reply carried (`null` if it had none).
        actual: Value,
    },
}

impl Error {
    /// Whether this is a transport problem rather than an RPC error reply.
    #[must_use]
    pub const fn is_transport(&self) -> bool {
        !matches!(self, Self::Rpc { .. })
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rpc { code, message, .. } => write!(f, "{message} (code {code})"),
            Self::BinaryNotFound => f.write_str(
                "sloptopop binary not found: pass a path in Options, set SLOPTOPOP_BIN, \
                 or put `sloptopop` on PATH (cargo install sloptopop-cli)",
            ),
            Self::Spawn { path, source } => {
                write!(f, "could not start {}: {source}", path.display())
            }
            Self::ProcessDied => f.write_str(
                "the sloptopop process closed its output before replying (see its stderr)",
            ),
            Self::Io(e) => write!(f, "i/o error talking to sloptopop: {e}"),
            Self::Framing(e) => write!(f, "framing error from sloptopop: {e}"),
            Self::InvalidReply(reason) => write!(f, "invalid reply from sloptopop: {reason}"),
            Self::UnexpectedId { expected, actual } => write!(
                f,
                "expected the reply to request {expected}, got one with id {actual}"
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Spawn { source, .. } => Some(source),
            Self::Io(e) => Some(e),
            Self::Framing(e) => Some(e),
            Self::Rpc { .. }
            | Self::BinaryNotFound
            | Self::ProcessDied
            | Self::InvalidReply(_)
            | Self::UnexpectedId { .. } => None,
        }
    }
}

impl From<FrameError> for Error {
    fn from(e: FrameError) -> Self {
        match e {
            FrameError::Io(e) => Self::Io(e),
            other @ (FrameError::InvalidHeader(_)
            | FrameError::MissingContentLength
            | FrameError::TooLarge(_)) => Self::Framing(other),
        }
    }
}

#[cfg(test)]
mod tests;
