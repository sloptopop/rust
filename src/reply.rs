//! Interpreting the frames the server sends: replies to this client's
//! requests, and messages of its own that the client skips.

use serde::Deserialize;
use serde_json::{Map, Value};

use crate::error::Error;

/// Interprets one incoming frame: `None` for a message from the server
/// (it has a `method`), otherwise the outcome of request `id`.
pub(crate) fn parse_reply(body: &[u8], id: u64) -> Option<Result<Value, Error>> {
    let object = match serde_json::from_slice::<Value>(body) {
        Ok(Value::Object(object)) => object,
        Ok(other) => {
            return Some(Err(Error::InvalidReply(format!(
                "expected a JSON object, got {other}"
            ))));
        }
        Err(e) => return Some(Err(Error::InvalidReply(format!("not JSON: {e}")))),
    };
    if object.contains_key("method") {
        return None;
    }
    Some(reply_value(object, id))
}

/// The `error` member of a reply.
#[derive(Deserialize)]
struct ErrorObject {
    code: i64,
    message: String,
    data: Option<Value>,
}

/// The result of a reply that must answer request `id`.
fn reply_value(mut reply: Map<String, Value>, id: u64) -> Result<Value, Error> {
    let actual = reply.remove("id").unwrap_or(Value::Null);
    if actual != id {
        return Err(Error::UnexpectedId {
            expected: id,
            actual,
        });
    }
    if let Some(error) = reply.remove("error") {
        let error = ErrorObject::deserialize(error)
            .map_err(|e| Error::InvalidReply(format!("malformed error object: {e}")))?;
        return Err(Error::Rpc {
            code: error.code,
            message: error.message,
            data: error.data,
        });
    }
    reply
        .remove("result")
        .ok_or_else(|| Error::InvalidReply("reply has neither result nor error".to_owned()))
}
