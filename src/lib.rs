//! Rust client for the sloptopop engine.
//!
//! Like every sloptopop language library, this crate starts the `sloptopop`
//! binary as `sloptopop serve --stdio` and talks JSON-RPC 2.0 to it over the
//! child's stdin/stdout (see `PROTOCOL.md` in the CLI repository). A future
//! version will also expose the engine in-process, since both are Rust; for
//! now it is the same process-based client as every other language.
//!
//! ```no_run
//! use sloptopop::{Client, Error, Options, code};
//! use serde_json::json;
//!
//! let mut client = Client::start(Options::default())?;
//! let info = client.server_info();
//! assert_eq!(info.server_name, "sloptopop");
//!
//! match client.request("nope", json!(null)) {
//!     Err(Error::Rpc { code, .. }) if code == code::METHOD_NOT_FOUND => {}
//!     other => panic!("unexpected {other:?}"),
//! }
//! assert!(client.close()?.success());
//! # Ok::<(), Error>(())
//! ```
//!
//! # Finding the binary
//!
//! First match wins:
//!
//! 1. [`Options::binary`], used as given;
//! 2. the `SLOPTOPOP_BIN` environment variable, if set and non-empty;
//! 3. a bundled binary: an executable `sloptopop` in the same directory as
//!    the current executable (Rust packages can't ship binaries, so put it
//!    there yourself if you distribute one);
//! 4. the first executable `sloptopop` on `PATH`, e.g. from
//!    `cargo install sloptopop-cli`.
//!
//! # Errors
//!
//! [`Error::Rpc`] carries the server's `code`, `message` and `data`; the
//! session stays usable. Every other [`Error`] variant is a transport
//! problem ([`Error::is_transport`]) after which the process is killed.
//!
//! # Module layers
//!
//! Each module imports only from modules in lower layers, never from its
//! own layer or above; this file only declares and re-exports them.
//!
//! 0. [`code`] (error codes), [`framing`] (`Content-Length` frames),
//!    `locate` (finding the binary), `options` ([`Options`]), `request`
//!    (outgoing messages): no crate imports.
//! 1. `error` ([`Error`]): `framing`.
//! 2. `handshake` (`initialize`, [`ServerInfo`]): `error`; `reply` (incoming
//!    messages): `error`.
//! 3. `rpc` (one request/reply exchange over a stream pair): `error`,
//!    `framing`, `reply`, `request`.
//! 4. `client` ([`Client`], the process): `error`, `handshake`, `locate`,
//!    `options`, `rpc`.

// Layer 0: no crate imports.
pub mod code;
pub mod framing;
mod locate;
mod options;
mod request;
// Layer 1.
mod error;
// Layer 2.
mod handshake;
mod reply;
// Layer 3.
mod rpc;
// Layer 4.
mod client;

pub use client::Client;
pub use error::Error;
pub use handshake::{CLIENT_NAME, CLIENT_VERSION, PROTOCOL_VERSION, ServerInfo};
pub use options::Options;
pub use serde_json::Value;
