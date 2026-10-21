//! Starts a sloptopop client, prints what the server reports, and closes
//! the session.
//!
//! ```text
//! cargo run --example hello
//! ```
//!
//! The binary is looked up as usual: `SLOPTOPOP_BIN`, next to this
//! executable, or `sloptopop` on `PATH` (`cargo install sloptopop-cli`).

use std::error::Error;
use std::io::{self, Write};

use sloptopop::{Client, Options};

/// Starts a client with `options`, writes the server's name, version and
/// protocol version to `out`, and closes the client.
pub(crate) fn run(options: Options, out: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    let client = Client::start(options)?;
    let info = client.server_info();
    writeln!(out, "server: {} {}", info.server_name, info.server_version)?;
    writeln!(out, "protocol: {}", info.protocol_version)?;
    client.close()?;
    Ok(())
}

#[cfg_attr(
    test,
    expect(dead_code, reason = "the integration tests call `run` directly")
)]
fn main() -> Result<(), Box<dyn Error>> {
    run(Options::default(), &mut io::stdout().lock())
}
