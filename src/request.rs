//! The JSON-RPC messages this client sends: requests and notifications.

use serde_json::{Map, Value, json};

/// Request `id` calling `method`; a `null` `params` is omitted.
pub(crate) fn request(id: u64, method: &str, params: Value) -> Value {
    let mut request = Map::new();
    request.insert("jsonrpc".to_owned(), json!("2.0"));
    request.insert("id".to_owned(), json!(id));
    request.insert("method".to_owned(), json!(method));
    if !params.is_null() {
        request.insert("params".to_owned(), params);
    }
    Value::Object(request)
}

/// A notification of `method`, without params.
pub(crate) fn notification(method: &str) -> Value {
    json!({ "jsonrpc": "2.0", "method": method })
}

#[cfg(test)]
mod tests;
