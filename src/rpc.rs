//! The JSON-RPC exchange over a pair of byte streams: sending framed
//! messages and waiting for the reply to a request.

use std::io::{BufRead, Write};

use serde_json::Value;

use crate::error::Error;
use crate::framing::{read_frame, write_frame};
use crate::reply::parse_reply;
use crate::request::{notification, request};

/// Writes one message.
fn send(writer: &mut dyn Write, message: &Value) -> Result<(), Error> {
    write_frame(writer, message.to_string().as_bytes()).map_err(Error::from)
}

/// Sends the notification `method`.
pub(crate) fn notify(writer: &mut dyn Write, method: &str) -> Result<(), Error> {
    send(writer, &notification(method))
}

/// Sends request `id` and returns its result, skipping server notifications.
pub(crate) fn call(
    writer: &mut dyn Write,
    reader: &mut dyn BufRead,
    id: u64,
    method: &str,
    params: Value,
) -> Result<Value, Error> {
    send(writer, &request(id, method, params))?;
    loop {
        let body = read_frame(reader)?.ok_or(Error::ProcessDied)?;
        if let Some(reply) = parse_reply(&body, id) {
            return reply;
        }
    }
}

#[cfg(test)]
mod tests;
