//! Every case in the CLI repository's `conformance/cases`, sent to and read
//! from the real binary with this crate's framing code.

use std::fs;
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde_json::Value;
use sloptopop::framing::{read_frame, write_frame};
use sloptopop::{Client, code};

use crate::support::{binary, cli_dir, options};

/// Whether `actual` matches `expected` with the suite's placeholders.
fn matches(expected: &Value, actual: &Value, version: &str) -> bool {
    match (expected, actual) {
        (Value::String(e), Value::String(a)) if e == "$SERVER_VERSION" => a == version,
        (Value::Object(e), Value::Object(a)) => {
            if let Some(Value::String(prefix)) = e.get("messagePrefix") {
                let rest_matches = e
                    .iter()
                    .filter(|(k, _)| *k != "messagePrefix")
                    .all(|(k, v)| a.get(k).is_some_and(|av| matches(v, av, version)));
                let others = a.keys().all(|k| k == "message" || e.contains_key(k));
                return rest_matches
                    && others
                    && a.get("message")
                        .and_then(Value::as_str)
                        .is_some_and(|m| m.starts_with(prefix.as_str()));
            }
            e.len() == a.len()
                && e.iter()
                    .all(|(k, v)| a.get(k).is_some_and(|av| matches(v, av, version)))
        }
        (Value::Array(e), Value::Array(a)) => {
            e.len() == a.len() && e.iter().zip(a).all(|(ev, av)| matches(ev, av, version))
        }
        _ => expected == actual,
    }
}

fn run_case(name: &str, case: &Value, version: &str) {
    let mut child = Command::new(binary())
        .args(["serve", "--stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("binary should start");
    let mut stdin = child.stdin.take().expect("piped");
    for message in case["input"].as_array().expect("input array") {
        let body = match message.get("$raw") {
            Some(Value::String(raw)) => raw.clone(),
            _ => message.to_string(),
        };
        write_frame(&mut stdin, body.as_bytes()).expect("write frame");
    }
    stdin.flush().expect("flush");
    drop(stdin);

    let mut stdout = BufReader::new(child.stdout.take().expect("piped"));
    let mut replies = Vec::new();
    while let Some(body) = read_frame(&mut stdout).expect("well-formed frame") {
        replies.push(serde_json::from_slice::<Value>(&body).expect("JSON reply"));
    }
    let status = child.wait().expect("wait");

    let expected = case["output"].as_array().expect("output array");
    assert_eq!(
        replies.len(),
        expected.len(),
        "{name}: reply count {replies:?}"
    );
    for (want, got) in expected.iter().zip(&replies) {
        assert!(
            matches(want, got, version),
            "{name}: expected {want}, got {got}"
        );
    }
    assert_eq!(
        status.code(),
        case["exitCode"]
            .as_i64()
            .and_then(|c| i32::try_from(c).ok()),
        "{name}: exit code"
    );
}

/// The conformance case files, sorted by path.
// det-expect(clippy::disallowed_methods): the test's one directory read; the paths are sorted before use
fn cases() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = fs::read_dir(cli_dir().join("conformance/cases"))
        .expect("cases directory")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .collect();
    paths.sort();
    paths
}

#[test]
fn every_conformance_case_passes() {
    let client = Client::start(options()).expect("client should start");
    let version = client.server_info().server_version.clone();
    client.close().expect("close");

    let paths = cases();
    assert!(paths.len() >= 14, "found {} cases", paths.len());
    for path in paths {
        let case: Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read case")).expect("JSON");
        run_case(&path.display().to_string(), &case, &version);
    }
}

#[test]
fn placeholder_matching() {
    let v = "1.0.0";
    let p = |s: &str| serde_json::from_str::<Value>(s).expect("JSON");
    assert!(
        matches(&p(r#""$SERVER_VERSION""#), &p(r#""1.0.0""#), v),
        "version"
    );
    assert!(
        !matches(&p(r#""$SERVER_VERSION""#), &p(r#""2""#), v),
        "other version"
    );
    let prefix = p(r#"{"code":1,"messagePrefix":"parse error: "}"#);
    assert!(
        matches(&prefix, &p(r#"{"code":1,"message":"parse error: x"}"#), v),
        "prefix"
    );
    assert!(
        !matches(&prefix, &p(r#"{"code":1,"message":"nope"}"#), v),
        "wrong prefix"
    );
    assert!(
        !matches(&prefix, &p(r#"{"code":2,"message":"parse error: x"}"#), v),
        "code"
    );
    assert!(
        !matches(
            &prefix,
            &p(r#"{"code":1,"message":"parse error: x","data":1}"#),
            v
        ),
        "extra member"
    );
    assert!(!matches(&p("[1]"), &p("[1,2]"), v), "array length");
    assert!(
        !matches(&p(r#"{"a":1}"#), &p(r#"{"a":1,"b":2}"#), v),
        "object size"
    );
}

#[test]
fn error_codes_match_the_schema() {
    let schema: Value = serde_json::from_str(
        &fs::read_to_string(cli_dir().join("schema/protocol.json")).expect("schema"),
    )
    .expect("JSON");
    let codes = &schema["errorCodes"];
    for (name, value) in [
        ("PARSE_ERROR", code::PARSE_ERROR),
        ("INVALID_REQUEST", code::INVALID_REQUEST),
        ("METHOD_NOT_FOUND", code::METHOD_NOT_FOUND),
        ("INVALID_PARAMS", code::INVALID_PARAMS),
        (
            "UNSUPPORTED_PROTOCOL_VERSION",
            code::UNSUPPORTED_PROTOCOL_VERSION,
        ),
        ("SERVER_NOT_INITIALIZED", code::SERVER_NOT_INITIALIZED),
    ] {
        assert_eq!(codes[name].as_i64(), Some(value), "{name}");
    }
    assert_eq!(
        codes.as_object().map(serde_json::Map::len),
        Some(6),
        "no other codes"
    );
    assert_eq!(
        schema["protocolVersion"].as_u64(),
        Some(u64::from(sloptopop::PROTOCOL_VERSION)),
        "protocol version"
    );
}
