//! Unit tests for [`super`].

use super::*;

#[test]
fn requests_carry_id_method_and_params() {
    assert_eq!(
        request(7, "m", json!({"a": [1]})),
        json!({"jsonrpc": "2.0", "id": 7, "method": "m", "params": {"a": [1]}}),
        "with params"
    );
    assert_eq!(
        request(4, "shutdown", Value::Null),
        json!({"jsonrpc": "2.0", "id": 4, "method": "shutdown"}),
        "null params omitted"
    );
}

#[test]
fn notifications_have_no_id() {
    assert_eq!(
        notification("exit"),
        json!({"jsonrpc": "2.0", "method": "exit"}),
        "exit"
    );
}
