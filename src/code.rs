//! Standard JSON-RPC and sloptopop error codes, as listed in `PROTOCOL.md`
//! and `schema/protocol.json`. Compare them with
//! [`Error::Rpc`](crate::error::Error::Rpc)'s `code`.

/// The body is not valid JSON.
pub const PARSE_ERROR: i64 = -32700;
/// Not a request object, wrong `jsonrpc`, batch, second `initialize`, or
/// a request after `shutdown`.
pub const INVALID_REQUEST: i64 = -32600;
/// Unknown method.
pub const METHOD_NOT_FOUND: i64 = -32601;
/// Params missing or wrongly shaped.
pub const INVALID_PARAMS: i64 = -32602;
/// `initialize` with another protocol version.
pub const UNSUPPORTED_PROTOCOL_VERSION: i64 = -32001;
/// A request before `initialize`.
pub const SERVER_NOT_INITIALIZED: i64 = -32002;

#[cfg(test)]
mod tests;
