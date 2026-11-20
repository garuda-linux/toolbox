use rustix::fs::{Mode, OFlags, open};
use rustix::io::{FdFlags, fcntl_getfd};

use super::{keep_from_children, stdin_copy};

/// rustix opens without close-on-exec unless asked, like a descriptor a
/// caller leaves open on purpose or by mistake.
#[test]
fn an_inherited_descriptor_is_kept_from_children() {
    let inherited = open("/proc/self/status", OFlags::RDONLY, Mode::empty()).unwrap();

    assert!(!fcntl_getfd(&inherited).unwrap().contains(FdFlags::CLOEXEC));

    keep_from_children();

    assert!(fcntl_getfd(&inherited).unwrap().contains(FdFlags::CLOEXEC));
}

/// The copy is the request socket, which no program the helper starts may get.
#[test]
fn the_copy_of_stdin_is_kept_from_children() {
    let copy = stdin_copy().unwrap();

    assert!(fcntl_getfd(&copy).unwrap().contains(FdFlags::CLOEXEC));
}
