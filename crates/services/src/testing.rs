//! Waiting for a background job in tests, shared with the app's own
//! tests through `toolbox_gui::testing`.

use std::time::{Duration, Instant};

/// How long a test waits for a background job: long enough for a loaded
/// machine, short enough for a hung test.
pub const PATIENCE: Duration = Duration::from_secs(20);
const POLL_GAP: Duration = Duration::from_millis(5);

/// Calls `step` until it gives a value, a few milliseconds apart. Panics
/// with `what` once [`PATIENCE`] ran out, so a job that never ends says
/// which one it was.
///
/// # Panics
///
/// When `step` gave nothing within [`PATIENCE`].
pub fn poll<T>(what: &str, mut step: impl FnMut() -> Option<T>) -> T {
    let start = Instant::now();
    loop {
        if let Some(value) = step() {
            return value;
        }

        assert!(start.elapsed() < PATIENCE, "{what}");
        std::thread::sleep(POLL_GAP);
    }
}

/// # Panics
///
/// When `done` did not hold within [`PATIENCE`].
pub fn until(what: &str, mut done: impl FnMut() -> bool) {
    poll(what, || done().then_some(()));
}
