//! The software renderer's threads. Without a GPU, wgpu renders through
//! lavapipe, which starts a thread per core in every test process.
//! nextest runs a process per core, so a 32 core machine ran 1024
//! render threads and tests waited minutes for a turn.

#[cfg(test)]
mod tests;

use std::sync::Once;

/// Gives lavapipe one thread per process, unless the runner chose a
/// number. Runs once, before the first render state, while a test
/// process has no thread that reads the environment outside std's lock.
pub(super) fn one_thread() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        if std::env::var_os("LP_NUM_THREADS").is_some() {
            return;
        }

        #[expect(
            unsafe_code,
            clippy::disallowed_methods,
            reason = "lavapipe reads its thread count only from the environment"
        )]
        // SAFETY: set before wgpu or any foreign thread starts.
        unsafe {
            std::env::set_var("LP_NUM_THREADS", "1");
        }
    });
}
