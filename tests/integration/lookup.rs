//! Binary lookup through the real environment. Each test re-runs itself in
//! a child process with a controlled environment, since this process's
//! environment can't safely be changed while other tests run.

use std::env;
use std::process::Command;

use sloptopop::{Client, Error, Options};

use crate::support::binary;

const CHILD: &str = "SLOPTOPOP_RUST_LOOKUP_CHILD";

/// In the parent, runs test `name` in a child with `SLOPTOPOP_BIN` and
/// `PATH` set as given (`None` removes them) and returns `true` if it
/// passed. In the child, returns `false` so the caller runs its checks.
fn in_child(name: &str, sloptopop_bin: Option<&str>, path: Option<&str>) -> bool {
    if env::var_os(CHILD).is_some() {
        return false;
    }
    let mut command = Command::new(env::current_exe().expect("test binary"));
    command
        .args([name, "--exact", "--test-threads=1"])
        .env(CHILD, "1")
        .env_remove("SLOPTOPOP_BIN")
        .env_remove("PATH");
    if let Some(value) = sloptopop_bin {
        command.env("SLOPTOPOP_BIN", value);
    }
    if let Some(value) = path {
        command.env("PATH", value);
    }
    let output = command.output().expect("test binary should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains("1 passed"),
        "child {name} failed:\n{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    true
}

fn real() -> String {
    binary().to_str().expect("UTF-8 path").to_owned()
}

#[test]
fn nothing_to_find_is_binary_not_found() {
    if in_child("lookup::nothing_to_find_is_binary_not_found", None, None) {
        return;
    }
    let error = Client::start(Options::default()).expect_err("nothing to find");
    assert!(matches!(error, Error::BinaryNotFound), "{error:?}");
}

#[test]
fn sloptopop_bin_is_used() {
    if in_child("lookup::sloptopop_bin_is_used", Some(&real()), None) {
        return;
    }
    let client = Client::start(Options::default()).expect("found via SLOPTOPOP_BIN");
    assert!(client.close().expect("close").success(), "clean exit");
}

#[test]
fn path_is_searched() {
    let dir = binary()
        .canonicalize()
        .expect("real binary exists")
        .parent()
        .expect("has a directory")
        .to_str()
        .expect("UTF-8 path")
        .to_owned();
    if in_child("lookup::path_is_searched", Some(""), Some(&dir)) {
        return;
    }
    let client = Client::start(Options::default()).expect("found on PATH");
    assert!(client.close().expect("close").success(), "clean exit");
}
