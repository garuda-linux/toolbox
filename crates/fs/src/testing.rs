//! Helpers for the tests of this crate and of the crates that replace files
//! with it.

use rustix::process::{Uid, geteuid};
use rustix::thread::set_thread_res_uid;

/// A person's uid, as the app runs under.
const PERSON: u32 = 1000;

/// Runs `op` as a person when the tests run as root.
///
/// The modes of files and folders then deny it as they deny a user. Without
/// root `op` runs as it is. The folders above a tempdir must be readable by
/// others, as a root-owned tempdir is closed to a person.
#[expect(
    clippy::unwrap_used,
    clippy::missing_panics_doc,
    reason = "a test helper, a failed uid change must stop the test"
)]
pub fn as_person<T>(op: impl FnOnce() -> T) -> T {
    if !geteuid().is_root() {
        return op();
    }

    set_thread_res_uid(None, Some(Uid::from_raw(PERSON)), None).unwrap();

    let result = op();

    set_thread_res_uid(None, Some(Uid::ROOT), None).unwrap();

    result
}
