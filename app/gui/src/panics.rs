//! Panics reach the log: a thread that dies otherwise leaves only a line
//! on a terminal nobody may be watching.

#[cfg(test)]
mod tests;

use std::backtrace::Backtrace;
use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::panic::{Location, PanicHookInfo};
use std::path::{Path, PathBuf};

/// Logs every panic with its place and a backtrace, adds it to
/// `crash_file`, then runs the hook that was there before.
///
/// Rust's own hook prints to stderr, which nobody may read once the window
/// is gone, so the crash file is what a bug report can attach.
pub fn log_panics(crash_file: PathBuf) {
    let previous = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info: &PanicHookInfo<'_>| {
        let thread = std::thread::current();
        let text = format!(
            "{}\n{}\n",
            headline(thread.name(), info.location(), info.payload_as_str()),
            Backtrace::force_capture()
        );

        tracing::error!("{}", text.trim_end());
        if let Err(error) = record(&crash_file, &text) {
            // stderr is the last place left, and a failed write there has
            // nowhere to go.
            writeln!(
                std::io::stderr(),
                "the crash file {} was not written: {error}",
                crash_file.display()
            )
            .ok();
        }

        previous(info);
    }));
}

/// Adds `text` to `file`, readable by the owner alone: backtraces hold
/// the user's paths.
fn record(file: &Path, text: &str) -> std::io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt as _;

    if let Some(folder) = file.parent() {
        fs::create_dir_all(folder)?;
    }

    OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(file)?
        .write_all(text.as_bytes())
}

fn headline(
    thread: Option<&str>,
    location: Option<&Location<'_>>,
    message: Option<&str>,
) -> String {
    let thread = thread.unwrap_or("a thread without a name");
    let message = message.unwrap_or("no message");

    let place = location.map_or_else(String::new, |location| format!(" at {location}"));

    format!("panic in {thread}{place}: {message}")
}
