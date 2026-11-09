//! Descriptors the helper inherited from whoever started it. The sudo
//! fallback may leave any of the caller's files or sockets open, and every
//! program the helper starts as root would get them too.

#[cfg(test)]
mod tests;

use std::fs;
use std::os::fd::{BorrowedFd, RawFd};

use rustix::io::{FdFlags, fcntl_getfd, fcntl_setfd};

/// The first descriptor past stdin, stdout and stderr.
const FIRST_INHERITED: RawFd = 3;

/// Marks every inherited descriptor close-on-exec. Runs at the start of
/// `main`, before any thread starts, so nothing opens one meanwhile.
pub fn keep_from_children() {
    let listed: Vec<RawFd> = match fs::read_dir("/proc/self/fd") {
        Ok(listing) => listing
            .filter_map(Result::ok)
            .filter_map(|entry| entry.file_name().to_str()?.parse().ok())
            .filter(|fd| *fd >= FIRST_INHERITED)
            .collect(),
        Err(error) => {
            log::warn!("cannot list the inherited descriptors: {error}");
            return;
        }
    };

    for fd in listed {
        mark(fd);
    }
}

#[expect(
    unsafe_code,
    reason = "rustix reaches a descriptor known only by its number through `borrow_raw` alone"
)]
fn mark(fd: RawFd) {
    // SAFETY: `fd` was open when `/proc/self/fd` listed it, and no thread
    // runs yet to close it or reuse its number. The one that may be closed
    // is the listing's own, for which fcntl fails without touching memory.
    let borrowed = unsafe { BorrowedFd::borrow_raw(fd) };

    if let Ok(flags) = fcntl_getfd(borrowed) {
        fcntl_setfd(borrowed, flags | FdFlags::CLOEXEC).ok();
    }
}
