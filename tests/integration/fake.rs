//! The client against fake binaries: shell scripts that misbehave in
//! specific ways. Each one first reads the `Content-Length` line of the
//! `initialize` request, so it never exits before the client has written
//! it, and none waits for input forever, so a client that fails to kill a
//! misbehaving process cannot hang the tests.

use serde_json::json;
use sloptopop::framing::FrameError;
use sloptopop::{Client, Error, Options, code};

use crate::support::{TempDir, alive, script};

const INIT_OK: &str = r#"{"jsonrpc":"2.0","id":1,"result":{"serverName":"fake","serverVersion":"0","protocolVersion":1}}"#;

/// A shell command printing `body` as one frame.
fn reply(body: &str) -> String {
    format!(
        "printf 'Content-Length: {}\\r\\n\\r\\n%s' '{body}'",
        body.len()
    )
}

/// Starts a client on a fake binary that runs `commands` after reading the
/// first request line. The script is deleted once the process is running.
fn start(name: &str, commands: &[String]) -> Result<Client, Error> {
    let dir = TempDir::new(&format!("fake-{name}"));
    let body = format!("read -r _header\n{}", commands.join("\n"));
    Client::start(Options::default().binary(script(&dir, "sloptopop", &body)))
}

#[test]
fn process_that_exits_immediately() {
    let error = start("exits", &["exit 3".to_owned()]).expect_err("no reply");
    assert!(matches!(error, Error::ProcessDied), "{error:?}");
    assert!(error.is_transport(), "transport error");
}

#[test]
fn garbage_instead_of_a_frame() {
    let error = start("garbage", &["printf 'garbage\\n'".to_owned()]).expect_err("garbage");
    assert!(
        matches!(&error, Error::Framing(FrameError::InvalidHeader(h)) if h == "garbage\n"),
        "{error:?}"
    );
}

#[test]
fn reply_with_the_wrong_id() {
    let body = INIT_OK.replace("\"id\":1", "\"id\":99");
    let error = start("wrong-id", &[reply(&body)]).expect_err("wrong id");
    assert!(
        matches!(&error, Error::UnexpectedId { expected: 1, actual } if *actual == json!(99)),
        "{error:?}"
    );
}

#[test]
fn reply_that_is_not_json() {
    let error = start("not-json", &[reply("nope")]).expect_err("not JSON");
    assert!(
        matches!(&error, Error::InvalidReply(r) if r.starts_with("not JSON: ")),
        "{error:?}"
    );
}

#[test]
fn server_refusing_the_protocol_version() {
    let body = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32001,"message":"unsupported protocol version 1; this server speaks 2","data":{"supported":[2]}}}"#;
    let error = start("refuses", &[reply(body)]).expect_err("refused");
    assert!(
        matches!(
            &error,
            Error::Rpc { code, data: Some(data), .. }
                if *code == code::UNSUPPORTED_PROTOCOL_VERSION && *data == json!({"supported": [2]})
        ),
        "{error:?}"
    );
}

#[test]
fn server_speaking_another_protocol_version() {
    let body = INIT_OK.replace("\"protocolVersion\":1", "\"protocolVersion\":2");
    let error = start("v2", &[reply(&body)]).expect_err("other protocol");
    assert!(
        matches!(&error, Error::InvalidReply(r) if r == "server speaks protocol 2, this client 1"),
        "{error:?}"
    );
}

#[test]
fn close_sends_shutdown_and_exit_then_closes_stdin() {
    let shutdown = r#"{"jsonrpc":"2.0","id":2,"result":null}"#;
    // Exits 7 only if, by the time stdin closes, it got shutdown then exit.
    let check = r#"input=$(cat)
case "$input" in
  *'"id":2,"jsonrpc":"2.0","method":"shutdown"}'*'{"jsonrpc":"2.0","method":"exit"}') exit 7 ;;
esac
exit 9"#;
    let client = start(
        "exit-7",
        &[reply(INIT_OK), reply(shutdown), check.to_owned()],
    )
    .expect("fake initializes");
    assert_eq!(client.server_info().server_name, "fake", "fake server");
    let pid = client.id();
    let status = client.close().expect("close");
    assert_eq!(status.code(), Some(7), "the fake saw shutdown and exit");
    assert!(!alive(pid), "waited for");
}

#[test]
fn transport_error_kills_the_process() {
    let wrong = r#"{"jsonrpc":"2.0","id":5,"result":null}"#;
    let mut client = start(
        "mid-session",
        &[
            reply(INIT_OK),
            reply(wrong),
            // The rest of the `initialize` frame is a blank line and a body
            // without a newline; the line after it is the next request's.
            "read -r _blank_line; read -r _next_header".to_owned(),
        ],
    )
    .expect("initializes");
    let pid = client.id();
    let error = client.request("x", json!(null)).expect_err("wrong id");
    assert!(
        matches!(&error, Error::UnexpectedId { expected: 2, .. }),
        "{error:?}"
    );
    // The fake exits by itself here; unless the client waited for it, it
    // would linger as a zombie.
    assert!(!alive(pid), "killed and waited for");
    let after = client.request("y", json!(null)).expect_err("dead");
    assert!(matches!(after, Error::ProcessDied), "{after:?}");
    let closed = client.close().expect_err("already dead");
    assert!(matches!(closed, Error::ProcessDied), "{closed:?}");
}

#[test]
fn binary_that_is_not_executable() {
    let dir = TempDir::new("not-executable");
    let path = dir.join("sloptopop");
    std::fs::write(&path, "#!/bin/sh\n").expect("write");
    let error = Client::start(Options::default().binary(&path)).expect_err("not executable");
    assert!(
        matches!(&error, Error::Spawn { source, .. } if source.kind() == std::io::ErrorKind::PermissionDenied),
        "{error:?}"
    );
}
