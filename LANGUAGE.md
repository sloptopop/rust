# Rust specifics

Only what is specific to the Rust client. The protocol, the CLI, the
conformance suite and the quality standard are documented once, in
[sloptopop/cli](https://github.com/sloptopop/cli)
([PROTOCOL.md](https://github.com/sloptopop/cli/blob/main/PROTOCOL.md),
[docs/](https://github.com/sloptopop/cli/tree/main/docs),
[conformance/](https://github.com/sloptopop/cli/tree/main/conformance)).

## Shared policy

Every sloptopop Rust repository uses the same policy files, kept in
`policy/` and installed with `policy/sync.sh`: the `[lints]` tables in
`Cargo.toml`, `clippy.toml`, `rust-toolchain.toml`, `.cargo/config.toml`,
`deny.toml`, `build.rs`, `scripts/gate.sh`, `.githooks/pre-push` and
`.github/workflows/gate.yml`. They are not edited per repository.

## Tools per standard item

| # | Item | Rust tooling (version, licence) |
|---|---|---|
| 1 | Pinned toolchain | Rust **1.99.0** in `rust-toolchain.toml` (MIT OR Apache-2.0); rustup installs exactly that. `Cargo.lock` pins every dependency and the gate passes `--locked` everywhere. |
| 2 | Formatting | `cargo fmt --check` (rustfmt, part of the toolchain). |
| 3 | Zero warnings | `warnings = "deny"` plus `clippy::all` and `clippy::pedantic` at deny, with the restriction lints in `Cargo.toml`. `.cargo/config.toml` makes Clippy the compiler wrapper, so `cargo build`, `check` and `test` fail on any lint. Suppressions are `#[expect(lint, reason = "...")]`, never `#[allow]`. |
| 4 | All tests pass | `cargo test`: unit tests in `src/<module>/tests.rs` (and `src/tests.rs` for `lib.rs`), `tests/integration/` against the real binary (`SLOPTOPOP_BIN`, default `../cli/target/debug/sloptopop`), every `../cli/conformance/cases/*.json` through this crate's own framing code, fake binaries (shell scripts) for transport errors, and `examples/hello.rs` run in-process. |
| 5 | 100% coverage | cargo-llvm-cov 0.9.1 (MIT OR Apache-2.0): **lines, regions and functions** at 100%, zero uncovered. Test code is excluded by file name: `tests.rs`, `*_tests.rs`, `tests/` and `examples/`. `build.rs` runs the same check on every `cargo build`. |
| 6 | Mutation testing | cargo-mutants 27.1.0 (MIT): every mutant must be caught; a timeout counts as caught. |
| 7 | Vulnerabilities | cargo-deny 0.20.2 (MIT OR Apache-2.0) `advisories`: RustSec vulnerabilities, yanked, unmaintained and unsound crates all fail. |
| 8 | Licences | cargo-deny `licenses` against the allow-list in `deny.toml`, no exceptions. Runtime dependencies: `serde` and `serde_json` only (their trees are MIT OR Apache-2.0, plus `unicode-ident` under Unicode-3.0). |
| 9 | Docs | `missing_docs` and the `rustdoc` lints at deny; `cargo doc` with `-D warnings`; doctests run. |
| 10 | Pre-push hook | `.githooks/pre-push` runs `scripts/gate.sh`; enable with `git config core.hooksPath .githooks`. |
| 11 | CI | `.github/workflows/gate.yml` runs the full gate on the pinned toolchain. |

## Running the gate

```sh
scripts/gate.sh          # everything (pre-push, CI); mutation testing takes minutes
scripts/gate.sh --fast   # fmt, build, tests + coverage, doctests
```

Needs cargo-llvm-cov, cargo-deny and cargo-mutants on `PATH`; a missing tool
fails the gate. The integration tests need the CLI repository built next to
this one (`cargo build` in `../cli`) or `SLOPTOPOP_BIN` pointing at a binary;
the conformance cases and `schema/protocol.json` are read from `../cli`.
`CARGO_BUILD_JOBS` and `CARGO_MUTANTS_JOBS` cap parallelism.

## Build-time coverage enforcement

`build.rs` runs `cargo llvm-cov` in a separate target directory before every
`cargo build`, `check`, `test` or `doc`; the crate does not compile unless
every test passes at 100% line, region and function coverage. It is skipped
only inside the coverage run itself and inside cargo-mutants' scratch copies,
where failing tests are the point.

## Where the binary goes

Cargo packages cannot ship native binaries. The `sloptopop` binary is looked
up in this order, first match wins:

1. `Options::default().binary(path)`, used as given.
2. `$SLOPTOPOP_BIN`, if set and non-empty, used as given.
3. An executable `sloptopop` in the same directory as the current executable,
   the Rust stand-in for a bundled binary.
4. The first executable `sloptopop` on `PATH`.

Rust users get the `sloptopop` command from the CLI crate itself,
`cargo install sloptopop-cli`; this crate has no second binary.

## Platform limitations

- Tests that use shell scripts as fake binaries, and the lookup tests, are
  Unix only; coverage is enforced on Unix.
- Requests block until the reply arrives; there are no timeouts.
- The server's stderr is inherited, not captured.
