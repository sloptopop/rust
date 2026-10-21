//! Integration tests: the client against the real `sloptopop` binary and
//! against fake binaries, the conformance cases through this crate's
//! framing code, binary lookup, and the example program.
//!
//! The real binary is `SLOPTOPOP_BIN` if set, otherwise the CLI repository's
//! debug build next to this repository (`../cli/target/debug/sloptopop`).

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "../../examples/hello.rs"]
mod hello;

// Every module is `cfg(test)` so Clippy's test allowances (`expect`,
// `panic`, indexing) apply to helpers as well as to `#[test]` functions.
#[cfg(test)]
mod conformance;
#[cfg(test)]
mod example;
#[cfg(all(test, unix))]
mod fake;
#[cfg(test)]
mod lookup;
#[cfg(test)]
mod real;
