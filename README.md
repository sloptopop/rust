# Slop To Pop Rust Package

Rust client for Slop To Pop. It runs the `sloptopop` binary
(`sloptopop serve --stdio`) and talks to it over JSON-RPC; dependencies are
`serde` and `serde_json` only. It is the same process-based client as
every other language.

- Protocol, CLI and conformance cases: [sloptopop/cli](https://github.com/sloptopop/cli)
  ([PROTOCOL.md](https://github.com/sloptopop/cli/blob/main/PROTOCOL.md),
  [docs/](https://github.com/sloptopop/cli/tree/main/docs)).
- Rust-specific tooling, quality gate, binary location and limitations:
  [LANGUAGE.md](LANGUAGE.md).

## Install

```toml
[dependencies]
sloptopop = "0.1"
serde_json = "1"
```

The `sloptopop` command is the CLI crate itself: `cargo install sloptopop-cli`.
This crate ships no second binary; it finds the installed one (or set
`SLOPTOPOP_BIN`, see [LANGUAGE.md](LANGUAGE.md#where-the-binary-goes)).

## Usage

```rust
use serde_json::json;
use sloptopop::{Client, Error, Options, code};

let mut client = Client::start(Options::default())?; // spawn + initialize
let info = client.server_info(); // server_name, server_version, protocol_version

match client.request("someMethod", json!({"key": 1})) {
    Ok(result) => { /* serde_json::Value */ }
    Err(Error::Rpc { code, message, data }) => { /* server error reply */ }
    Err(transport) => { /* binary missing, process died, framing, unexpected id */ }
}

client.close()?; // shutdown, exit, wait; Drop does the same best-effort
```

`Options::default().binary(path)` picks the binary. The framing codec is
exported as `sloptopop::framing::{read_frame, write_frame}`.

## Example

```sh
$ cargo run --example hello
server: sloptopop 0.1.0
protocol: 1
```

See [examples/hello.rs](examples/hello.rs).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
