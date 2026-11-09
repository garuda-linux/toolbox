use std::io;

use crate::capture::process::whole;
use crate::capture::{CaptureError, Tail};

#[test]
fn a_pipe_that_broke_midway_is_an_error_and_not_a_short_output() {
    let mut tail = Tail::new(64);

    tail.push(b"libfoo\n");
    tail.failed = Some(io::Error::other("Input/output error"));

    assert!(matches!(whole(tail), Err(CaptureError::Failed(_))));
}

#[test]
fn a_pipe_read_to_its_end_is_the_whole_output() {
    let mut tail = Tail::new(64);

    tail.push(b"libfoo\n");

    assert_eq!(whole(tail).unwrap().into_bytes().0, b"libfoo\n");
}
