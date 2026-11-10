use super::{read, read_optional};
use crate::ProbeError;

const PASSWD: &str = toolbox_plan::testing::accounts::PASSWD;

/// The Lima VM's `passwd` with a Latin-1 byte as root's full name.
#[test]
fn a_byte_that_is_no_utf8_does_not_fail_the_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("passwd");

    let (root, rest) = PASSWD.split_once("::/root").unwrap();

    std::fs::write(
        &path,
        [root.as_bytes(), b":\xFF:/root", rest.as_bytes()].concat(),
    )
    .unwrap();

    let text = read(&path).unwrap();

    assert_eq!(text.lines().count(), PASSWD.lines().count());
    assert!(
        text.starts_with("root:x:0:0:\u{fffd}:/root:/usr/bin/bash"),
        "{text}"
    );
    assert_eq!(read_optional(&path).unwrap(), Some(text));
}

#[test]
fn a_missing_file_fails_unless_it_is_optional() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("absent");

    assert_eq!(read(&path), Err(ProbeError::Unavailable));
    assert_eq!(read_optional(&path).unwrap(), None);
}
