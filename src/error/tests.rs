//! Unit tests for [`super`].

use std::error::Error as _;

use serde_json::json;

use super::*;
use crate::code;

fn every_variant() -> Vec<(Error, &'static str, bool)> {
    vec![
        (
            Error::Rpc {
                code: code::METHOD_NOT_FOUND,
                message: "method not found: x".into(),
                data: None,
            },
            "method not found: x (code -32601)",
            false,
        ),
        (Error::BinaryNotFound, "sloptopop binary not found", false),
        (
            Error::Spawn {
                path: PathBuf::from("/nowhere/sloptopop"),
                source: io::ErrorKind::NotFound.into(),
            },
            "could not start /nowhere/sloptopop: ",
            true,
        ),
        (
            Error::ProcessDied,
            "the sloptopop process closed its output",
            false,
        ),
        (
            Error::Io(io::ErrorKind::BrokenPipe.into()),
            "i/o error talking to sloptopop: ",
            true,
        ),
        (
            Error::Framing(FrameError::MissingContentLength),
            "framing error from sloptopop: missing",
            true,
        ),
        (
            Error::InvalidReply("bad".into()),
            "invalid reply from sloptopop: bad",
            false,
        ),
        (
            Error::UnexpectedId {
                expected: 3,
                actual: json!(9),
            },
            "expected the reply to request 3, got one with id 9",
            false,
        ),
    ]
}

#[test]
fn errors_describe_themselves() {
    for (error, expected, _) in every_variant() {
        assert!(error.to_string().starts_with(expected), "{error}");
    }
}

#[test]
fn wrapped_errors_are_the_source() {
    for (error, _, has_source) in every_variant() {
        assert_eq!(error.source().is_some(), has_source, "{error}");
    }
}

#[test]
fn only_rpc_errors_are_not_transport_errors() {
    for (error, _, _) in every_variant() {
        let is_rpc = matches!(error, Error::Rpc { .. });
        assert_eq!(error.is_transport(), !is_rpc, "{error}");
    }
}

#[test]
fn frame_io_errors_become_io_errors() {
    let error = Error::from(FrameError::Io(io::ErrorKind::UnexpectedEof.into()));
    assert!(
        matches!(error, Error::Io(e) if e.kind() == io::ErrorKind::UnexpectedEof),
        "io"
    );
}

#[test]
fn other_frame_errors_become_framing_errors() {
    for frame_error in [
        FrameError::InvalidHeader("h".into()),
        FrameError::MissingContentLength,
        FrameError::TooLarge(1),
    ] {
        let text = frame_error.to_string();
        let error = Error::from(frame_error);
        assert!(
            matches!(&error, Error::Framing(e) if e.to_string() == text),
            "{error}"
        );
    }
}
