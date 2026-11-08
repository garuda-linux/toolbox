use super::{read, read_optional};
use crate::ProbeError;

#[test]
fn a_byte_that_is_no_utf8_does_not_fail_the_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("passwd");

    std::fs::write(
        &path,
        b"root:x:0:0:\xFF:/root:/bin/bash\nnico:x:1000:1000::/home/nico:/bin/fish\n",
    )
    .unwrap();

    let text = read(&path).unwrap();

    assert_eq!(text.lines().count(), 2);
    assert!(text.starts_with("root:x:0:0:\u{fffd}:/root"), "{text}");
    assert_eq!(read_optional(&path).unwrap(), Some(text));
}

#[test]
fn a_missing_file_fails_unless_it_is_optional() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("absent");

    assert_eq!(read(&path), Err(ProbeError::Unavailable));
    assert_eq!(read_optional(&path).unwrap(), None);
}
