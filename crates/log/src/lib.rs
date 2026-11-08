//! Permanent, rotating log file for the app: `<folder>/<program>.log` (plus
//! `.1`…`.3`) in the folder the caller names.
//!
//! The app logs through tracing ([`subscriber`]). The level comes from
//! `[general] log_level` and can change while running. Records are also
//! echoed to stderr, so a program started in a terminal shows its log there.

pub mod calendar;
mod file;
mod format;
pub mod subscriber;
mod terminal;

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use file::FileLogger;

pub(crate) static LOGGER: OnceLock<FileLogger> = OnceLock::new();

/// Keeps the log out of the terminal, for a command whose own output is
/// the answer. The file still gets every line.
pub fn quiet_terminal() {
    if let Some(logger) = LOGGER.get() {
        logger.quiet_terminal();
    }
}

/// Opens `<dir>/<program>.log` and writes the buffered lines first. Call
/// once after the single-instance claim.
///
/// Returns the file path, or `None` when no logger was installed or the
/// caller has no safe folder (`dir` is `None`, the log then stays on
/// stderr). A file that cannot be opened only disables file output, so
/// logging never fails.
pub fn open_file(dir: Option<&Path>, program: &str) -> Option<PathBuf> {
    let logger = LOGGER.get()?;

    let Some(dir) = dir else {
        logger.stderr_only();
        return None;
    };

    let path = dir.join(format!("{program}.log"));

    logger.open_file(&path);

    // Past the filter, so every file says which filter it was written with.
    let started = format!(
        "{program} {} started ({})",
        env!("CARGO_PKG_VERSION"),
        subscriber::description()
    );

    logger.emit(log::Level::Info, module_path!(), &started);

    Some(path)
}
