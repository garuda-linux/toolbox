//! The helper tells who asks by the peer of its stdin, which only a
//! socket has.

use std::sync::Arc;

use super::{LIMIT, shell};
use crate::capture::{Policy, capture};
use crate::pty_runner::Environment;

fn stdin_kind(policy: &Policy<'_>) -> String {
    capture(
        &mut shell("stat -L -c %F /proc/self/fd/0"),
        Environment::Inherited,
        &[],
        policy,
        &Arc::default(),
    )
    .unwrap()
    .output
}

#[test]
fn an_identified_program_reads_from_a_socket() {
    assert_eq!(stdin_kind(&Policy::tail(LIMIT).identified()), "socket\n");
}

#[test]
fn any_other_program_reads_nothing() {
    assert_eq!(stdin_kind(&Policy::tail(LIMIT)), "character special file\n");
}
