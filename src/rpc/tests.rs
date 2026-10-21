//! Unit tests for [`super`], over in-memory streams; process handling is
//! tested against real and fake binaries in `tests/`.

use std::io;

use serde_json::json;

use super::*;

fn frame(body: &str) -> Vec<u8> {
    let mut out = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    out.extend_from_slice(body.as_bytes());
    out
}

fn frames(bodies: &[&str]) -> Vec<u8> {
    bodies.iter().flat_map(|b| frame(b)).collect()
}

/// Runs `call` for request `id` against `replies`; returns the outcome and
/// the bytes written.
fn call_with(
    replies: &[u8],
    id: u64,
    method: &str,
    params: Value,
) -> (Result<Value, Error>, String) {
    let mut written = Vec::new();
    let result = call(&mut written, &mut &replies[..], id, method, params);
    (result, String::from_utf8(written).expect("utf-8"))
}

#[test]
fn request_without_params_omits_them() {
    let replies = frame(r#"{"jsonrpc":"2.0","id":4,"result":null}"#);
    let (result, written) = call_with(&replies, 4, "shutdown", Value::Null);
    assert_eq!(result.ok(), Some(Value::Null), "null result");
    assert_eq!(
        written,
        String::from_utf8(frame(r#"{"id":4,"jsonrpc":"2.0","method":"shutdown"}"#)).expect("utf-8"),
        "request bytes"
    );
}

#[test]
fn request_with_params_sends_them() {
    let replies = frame(r#"{"jsonrpc":"2.0","id":7,"result":1}"#);
    let (result, written) = call_with(&replies, 7, "m", json!({"a": [1]}));
    assert_eq!(result.ok(), Some(json!(1)), "result");
    assert!(
        written.ends_with(r#"{"id":7,"jsonrpc":"2.0","method":"m","params":{"a":[1]}}"#),
        "{written}"
    );
}

#[test]
fn server_messages_are_skipped() {
    let replies = frames(&[
        r#"{"jsonrpc":"2.0","method":"$/progress"}"#,
        r#"{"jsonrpc":"2.0","id":9,"method":"window/ask"}"#,
        r#"{"jsonrpc":"2.0","id":1,"result":"ok"}"#,
    ]);
    let (result, _) = call_with(&replies, 1, "m", Value::Null);
    assert_eq!(result.ok(), Some(json!("ok")), "reply after notifications");
}

#[test]
fn rpc_errors_carry_code_message_and_data() {
    let replies = frame(
        r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32001,"message":"no","data":{"supported":[1]}}}"#,
    );
    let (result, _) = call_with(&replies, 1, "initialize", Value::Null);
    assert!(
        matches!(
            result,
            Err(Error::Rpc { code: -32001, ref message, data: Some(ref data) })
                if message == "no" && *data == json!({"supported": [1]})
        ),
        "{result:?}"
    );
}

#[test]
fn rpc_errors_without_data() {
    let replies = frame(r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"x"}}"#);
    let (result, _) = call_with(&replies, 1, "nope", Value::Null);
    assert!(
        matches!(
            result,
            Err(Error::Rpc {
                code: -32601,
                data: None,
                ..
            })
        ),
        "{result:?}"
    );
}

#[test]
fn malformed_error_objects_are_invalid_replies() {
    let replies = frame(r#"{"jsonrpc":"2.0","id":1,"error":{"code":"x"}}"#);
    let (result, _) = call_with(&replies, 1, "m", Value::Null);
    assert!(
        matches!(result, Err(Error::InvalidReply(ref r)) if r.starts_with("malformed error object: ")),
        "{result:?}"
    );
}

#[test]
fn reply_without_result_or_error_is_invalid() {
    let replies = frame(r#"{"jsonrpc":"2.0","id":1}"#);
    let (result, _) = call_with(&replies, 1, "m", Value::Null);
    assert!(
        matches!(result, Err(Error::InvalidReply(ref r)) if r == "reply has neither result nor error"),
        "{result:?}"
    );
}

#[test]
fn wrong_or_missing_ids_are_unexpected() {
    for (body, actual) in [
        (r#"{"jsonrpc":"2.0","id":2,"result":null}"#, json!(2)),
        (r#"{"jsonrpc":"2.0","id":"1","result":null}"#, json!("1")),
        (r#"{"jsonrpc":"2.0","result":null}"#, Value::Null),
        (
            r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"p"}}"#,
            Value::Null,
        ),
    ] {
        let (result, _) = call_with(&frame(body), 1, "m", Value::Null);
        assert!(
            matches!(result, Err(Error::UnexpectedId { expected: 1, actual: ref a }) if *a == actual),
            "{body}: {result:?}"
        );
    }
}

#[test]
fn non_object_and_non_json_bodies_are_invalid() {
    for (body, prefix) in [
        ("[1]", "expected a JSON object, got [1]"),
        ("{nope", "not JSON: "),
    ] {
        let (result, _) = call_with(&frame(body), 1, "m", Value::Null);
        assert!(
            matches!(result, Err(Error::InvalidReply(ref r)) if r.starts_with(prefix)),
            "{body}: {result:?}"
        );
    }
}

#[test]
fn end_of_output_means_the_process_died() {
    let (result, _) = call_with(b"", 1, "m", Value::Null);
    assert!(matches!(result, Err(Error::ProcessDied)), "{result:?}");
}

#[test]
fn bad_frames_are_framing_errors() {
    let (result, _) = call_with(b"garbage\n", 1, "m", Value::Null);
    assert!(
        matches!(
            result,
            Err(Error::Framing(FrameErrorAlias::InvalidHeader(_)))
        ),
        "{result:?}"
    );
    let (short, _) = call_with(b"Content-Length: 5\r\n\r\n{}", 1, "m", Value::Null);
    assert!(
        matches!(short, Err(Error::Io(ref e)) if e.kind() == io::ErrorKind::UnexpectedEof),
        "{short:?}"
    );
}

use crate::framing::FrameError as FrameErrorAlias;

/// A writer that always fails.
struct Broken;

impl Write for Broken {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::ErrorKind::BrokenPipe.into())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn write_failures_are_io_errors() {
    let replies = frame(r#"{"jsonrpc":"2.0","id":1,"result":null}"#);
    let result = call(&mut Broken, &mut &replies[..], 1, "m", Value::Null);
    assert!(
        matches!(result, Err(Error::Io(ref e)) if e.kind() == io::ErrorKind::BrokenPipe),
        "{result:?}"
    );
    assert!(
        matches!(send(&mut Broken, &json!({})), Err(Error::Io(_))),
        "send"
    );
}

#[test]
fn notifications_are_framed() {
    let mut written = Vec::new();
    assert!(notify(&mut written, "exit").is_ok(), "notify succeeds");
    assert_eq!(
        written,
        frame(r#"{"jsonrpc":"2.0","method":"exit"}"#),
        "notification bytes"
    );
    assert!(
        matches!(notify(&mut Broken, "exit"), Err(Error::Io(_))),
        "write failure"
    );
}
