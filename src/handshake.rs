//! The `initialize` handshake: what this client announces and which server
//! replies it accepts.

use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::Error;

/// `clientName` sent in `initialize`.
pub const CLIENT_NAME: &str = "sloptopop-rust";

/// `clientVersion` sent in `initialize`: this crate's version.
pub const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The protocol version this client speaks.
pub const PROTOCOL_VERSION: u32 = 1;

/// The server's reply to `initialize` (`InitializeResult` in the schema).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ServerInfo {
    /// Always `"sloptopop"`.
    pub server_name: String,
    /// Version of the binary.
    pub server_version: String,
    /// Protocol version the server speaks; always [`PROTOCOL_VERSION`] once
    /// [`Client::start`](crate::client::Client::start) has succeeded.
    pub protocol_version: u32,
}

/// The `params` of `initialize`.
pub(crate) fn initialize_params() -> Value {
    json!({
        "clientName": CLIENT_NAME,
        "clientVersion": CLIENT_VERSION,
        "protocolVersion": PROTOCOL_VERSION,
    })
}

/// Checks the `initialize` result.
pub(crate) fn server_info_from(result: Value) -> Result<ServerInfo, Error> {
    let info = ServerInfo::deserialize(result)
        .map_err(|e| Error::InvalidReply(format!("invalid initialize result: {e}")))?;
    if info.protocol_version == PROTOCOL_VERSION {
        Ok(info)
    } else {
        Err(Error::InvalidReply(format!(
            "server speaks protocol {}, this client {PROTOCOL_VERSION}",
            info.protocol_version
        )))
    }
}

#[cfg(test)]
mod tests;
