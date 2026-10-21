//! Unit tests for [`super`].

use super::*;

#[test]
fn codes_match_the_protocol() {
    assert_eq!(
        [
            PARSE_ERROR,
            INVALID_REQUEST,
            METHOD_NOT_FOUND,
            INVALID_PARAMS,
            UNSUPPORTED_PROTOCOL_VERSION,
            SERVER_NOT_INITIALIZED,
        ],
        [-32700, -32600, -32601, -32602, -32001, -32002],
        "PROTOCOL.md error table"
    );
}
