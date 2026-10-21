//! Unit tests for [`super`].

use super::*;

#[test]
fn options_builder_sets_the_binary() {
    assert_eq!(Options::default().binary, None, "default looks up");
    assert_eq!(
        Options::default().binary("/x/sloptopop").binary,
        Some(PathBuf::from("/x/sloptopop")),
        "explicit"
    );
}
