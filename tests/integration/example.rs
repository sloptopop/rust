//! The example program, run in-process with the real binary.

use crate::hello;
use crate::support::options;

#[test]
fn hello_prints_server_info() {
    let mut out = Vec::new();
    hello::run(options(), &mut out).expect("example should succeed");
    let text = String::from_utf8(out).expect("UTF-8 output");
    let mut lines = text.lines();
    assert!(
        lines
            .next()
            .is_some_and(|l| l.starts_with("server: sloptopop ")),
        "{text}"
    );
    assert_eq!(lines.next(), Some("protocol: 1"), "{text}");
    assert_eq!(lines.next(), None, "{text}");
}
