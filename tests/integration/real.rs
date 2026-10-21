//! The client against the real `sloptopop` binary.

use serde_json::json;
use sloptopop::{Client, Error, Options, PROTOCOL_VERSION, code};

use crate::support::{alive, binary, options};

#[test]
fn lifecycle_initialize_then_close() {
    let client = Client::start(options()).expect("client should start");
    let info = client.server_info().clone();
    assert_eq!(info.server_name, "sloptopop", "server name");
    assert!(!info.server_version.is_empty(), "server version");
    assert_eq!(info.protocol_version, PROTOCOL_VERSION, "protocol");
    let pid = client.id();
    assert!(alive(pid), "server running while the client is open");
    let status = client.close().expect("close should succeed");
    assert_eq!(status.code(), Some(0), "clean shutdown exits 0");
    assert!(!alive(pid), "server process was waited for");
}

#[test]
fn server_version_matches_the_binary() {
    let output = std::process::Command::new(binary())
        .arg("--version")
        .output()
        .expect("binary should run");
    let client = Client::start(options()).expect("client should start");
    let expected = format!(
        "sloptopop {} (protocol {PROTOCOL_VERSION})\n",
        client.server_info().server_version
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected,
        "version line"
    );
    client.close().expect("close");
}

#[test]
fn unknown_method_is_an_rpc_error_and_the_session_continues() {
    let mut client = Client::start(options()).expect("client should start");
    let error = client
        .request("nope", json!(null))
        .expect_err("unknown method");
    assert!(
        matches!(
            &error,
            Error::Rpc { code, message, data: None }
                if *code == code::METHOD_NOT_FOUND && message == "method not found: nope"
        ),
        "{error:?}"
    );
    assert!(
        !error.is_transport(),
        "an RPC error is not a transport error"
    );
    let again = client
        .request("still-nope", json!({"x": 1}))
        .expect_err("again");
    assert!(
        matches!(again, Error::Rpc { code, .. } if code == code::METHOD_NOT_FOUND),
        "{again:?}"
    );
    assert!(client.close().expect("close").success(), "clean exit");
}

#[test]
fn params_are_sent() {
    let mut client = Client::start(options()).expect("client should start");
    let error = client
        .request(
            "initialize",
            json!({"clientName": "x", "clientVersion": "0", "protocolVersion": 1}),
        )
        .expect_err("second initialize");
    assert!(
        matches!(
            &error,
            Error::Rpc { code, message, .. }
                if *code == code::INVALID_REQUEST && message == "initialize may only be sent once"
        ),
        "{error:?}"
    );
    let missing = client
        .request("initialize", json!(null))
        .expect_err("no params");
    assert!(
        matches!(&missing, Error::Rpc { code, .. } if *code == code::INVALID_REQUEST),
        "{missing:?}"
    );
    client.close().expect("close");
}

#[test]
fn shutdown_by_hand_makes_close_fail_but_still_reap() {
    let mut client = Client::start(options()).expect("client should start");
    let pid = client.id();
    assert_eq!(
        client.request("shutdown", json!(null)).ok(),
        Some(json!(null)),
        "shutdown"
    );
    let error = client.close().expect_err("second shutdown is refused");
    assert!(
        matches!(&error, Error::Rpc { code, .. } if *code == code::INVALID_REQUEST),
        "{error:?}"
    );
    assert!(!alive(pid), "process was waited for");
}

#[test]
fn drop_closes_the_process() {
    let client = Client::start(options()).expect("client should start");
    let pid = client.id();
    drop(client);
    assert!(!alive(pid), "drop waited for the process");
}

#[test]
fn debug_shows_the_process_and_server() {
    let client = Client::start(options()).expect("client should start");
    let text = format!("{client:?}");
    assert!(
        text.starts_with(&format!(
            "Client {{ id: {}, info: ServerInfo {{ server_name: \"sloptopop\"",
            client.id()
        )),
        "{text}"
    );
    assert!(text.contains("closed: false"), "{text}");
    client.close().expect("close");
}

#[test]
fn explicit_binary_that_does_not_exist() {
    let error = Client::start(Options::default().binary("/nonexistent/sloptopop"))
        .expect_err("no such binary");
    assert!(
        matches!(&error, Error::Spawn { path, source }
            if path.to_str() == Some("/nonexistent/sloptopop")
                && source.kind() == std::io::ErrorKind::NotFound),
        "{error:?}"
    );
}
