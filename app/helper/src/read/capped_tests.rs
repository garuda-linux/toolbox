use std::io::{self, Read};

use super::fill_capped;

/// Fails the way a disk does, after the bytes before it.
struct Broken;

impl Read for Broken {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("the disk went away"))
    }
}

#[test]
fn a_read_that_fails_midway_leaves_none_of_the_bytes_it_read() {
    let mut content = Vec::new();
    let source = (&b"root:$6$hash"[..]).chain(Broken);

    fill_capped(source, &mut content).unwrap_err();

    assert!(content.iter().all(|&byte| byte == 0), "{content:?}");
}
