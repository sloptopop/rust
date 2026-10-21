//! Unit tests for [`super`].

use super::*;

#[test]
fn initialize_params_announce_this_client() {
    assert_eq!(
        initialize_params(),
        json!({
            "clientName": "sloptopop-rust",
            "clientVersion": env!("CARGO_PKG_VERSION"),
            "protocolVersion": 1,
        }),
        "InitializeParams"
    );
}

#[test]
fn initialize_result_is_checked() {
    let good = json!({"serverName": "sloptopop", "serverVersion": "1.2.3", "protocolVersion": 1});
    assert_eq!(
        server_info_from(good).ok(),
        Some(ServerInfo {
            server_name: "sloptopop".into(),
            server_version: "1.2.3".into(),
            protocol_version: 1,
        }),
        "valid result"
    );
    let other = json!({"serverName": "sloptopop", "serverVersion": "9", "protocolVersion": 2});
    assert!(
        matches!(server_info_from(other), Err(Error::InvalidReply(ref r)) if r == "server speaks protocol 2, this client 1"),
        "other protocol"
    );
    assert!(
        matches!(server_info_from(json!(null)), Err(Error::InvalidReply(ref r)) if r.starts_with("invalid initialize result: ")),
        "wrong shape"
    );
}

#[test]
fn client_identity_constants() {
    assert_eq!(CLIENT_NAME, "sloptopop-rust", "PROTOCOL.md naming");
    assert_eq!(CLIENT_VERSION, env!("CARGO_PKG_VERSION"), "package version");
    assert_eq!(PROTOCOL_VERSION, 1, "protocol 1");
}
