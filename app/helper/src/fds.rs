//! Descriptors the helper inherited from whoever started it. A launcher
//! may leave any of the caller's files or sockets open, and every program
//! the helper starts as root would get them too.

#[cfg(test)]
mod tests;

use std::os::fd::RawFd;

/// The first descriptor past stdin, stdout and stderr.
const FIRST_INHERITED: RawFd = 3;

/// Marks every inherited descriptor close-on-exec. Runs at the start of
/// `main`, before any thread starts, so nothing opens one meanwhile.
pub fn keep_from_children() {
    close_fds::set_fds_cloexec(FIRST_INHERITED, &[]);
}
