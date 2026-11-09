//! The size-rotated log file behind the tracing subscriber.
//!
//! One `FileLogger` owns its file: every write takes the sink lock, and
//! rotation runs under that same lock, so no second writer can interleave
//! a rename with a line.

use std::collections::VecDeque;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime};

use parking_lot::Mutex;

use crate::format::{Style, format_record};
use crate::terminal::echo;

/// A log file is rotated once it grows past this size.
const MAX_SIZE: u64 = 5 * 1024 * 1024;
/// Rotated files kept next to the current one (`.1` is the newest).
const KEEP: u32 = 3;

/// A sink that could not be opened is tried again this much later: a
/// state folder that appears after the start still gets its log, and a
/// broken one does not cost an open per line.
const RETRY_AFTER: Duration = Duration::from_secs(30);

/// Lines kept while no file is open, so early startup notes survive until
/// the single-instance claim. Bounded, oldest goes first.
const BUFFERED_LINES: usize = 1000;

pub struct FileLogger {
    retry_after: Duration,
    // One lock for the path and the file: two locks taken in different
    // orders by `open_file` and `write_line` could deadlock.
    state: Mutex<State>,
    /// A command line prints its own answer, its log stays in the file.
    quiet: AtomicBool,
}

struct State {
    /// `None` until `open_file`: lines are buffered meanwhile.
    path: Option<PathBuf>,
    sink: Option<Sink>,
    /// When the last open failed.
    failed: Option<Instant>,
    buffered: VecDeque<String>,
    /// No file will ever open, so nothing is buffered either.
    stderr_only: bool,
    streak: Streak,
}

struct Sink {
    file: File,
    size: u64,
}

/// Lost lines since the last one that landed. A full disk fails every
/// line, so only the first failure of a streak reaches stderr.
#[derive(Default)]
struct Streak {
    failing: bool,
}

impl Streak {
    const fn failed(&mut self) -> bool {
        !std::mem::replace(&mut self.failing, true)
    }

    const fn recovered(&mut self) {
        self.failing = false;
    }
}

impl FileLogger {
    /// A logger with no file yet: lines are buffered in memory until
    /// [`Self::open_file`] runs after the single-instance claim.
    pub const fn buffered() -> Self {
        Self {
            retry_after: RETRY_AFTER,
            state: Mutex::new(State::empty()),
            quiet: AtomicBool::new(false),
        }
    }

    /// Drops the buffered lines and buffers no more. Records still reach
    /// stderr.
    pub fn stderr_only(&self) {
        let mut state = self.state.lock();
        state.stderr_only = true;
        state.buffered = VecDeque::new();
    }

    /// Opens the file and writes the buffered lines first. Later lines go
    /// straight to the file.
    pub fn open_file(&self, path: &Path) {
        let mut state = self.state.lock();

        state.path = Some(path.to_owned());
        state.reopen(path);
        let buffered = std::mem::take(&mut state.buffered);

        for line in buffered {
            state.write(&line);
        }
    }

    pub fn emit(&self, level: log::Level, target: &str, message: &str) {
        let time = SystemTime::now();

        self.write_line(&format_record(time, level, target, message, Style::Plain));
        if self.echoes() {
            echo(time, level, target, message);
        }
    }

    pub fn quiet_terminal(&self) {
        self.quiet.store(true, Ordering::Relaxed);
    }

    pub fn echoes(&self) -> bool {
        !self.quiet.load(Ordering::Relaxed)
    }

    fn write_line(&self, line: &str) {
        let mut state = self.state.lock();

        let Some(path) = state.path.clone() else {
            if state.stderr_only {
                return;
            }

            if state.buffered.len() >= BUFFERED_LINES {
                state.buffered.pop_front();
            }

            state.buffered.push_back(line.to_owned());
            return;
        };

        match state.sink.as_ref().map(|sink| sink.size >= MAX_SIZE) {
            Some(true) => {
                state.sink = None;
                rotate(&path);
                state.reopen(&path);
            }
            Some(false) => {}
            None => {
                if state
                    .failed
                    .is_none_or(|failed| failed.elapsed() >= self.retry_after)
                {
                    state.reopen(&path);
                }
            }
        }

        state.write(line);
    }
}

impl State {
    const fn empty() -> Self {
        Self {
            path: None,
            sink: None,
            failed: None,
            buffered: VecDeque::new(),
            stderr_only: false,
            streak: Streak { failing: false },
        }
    }

    fn reopen(&mut self, path: &Path) {
        match open_sink(path) {
            Ok(sink) => {
                self.sink = Some(sink);
                self.failed = None;
            }
            Err(error) => {
                self.sink = None;
                self.failed = Some(Instant::now());
                if self.streak.failed() {
                    report(&format!("cannot open the log {}: {error}", path.display()));
                }
            }
        }
    }

    fn write(&mut self, line: &str) {
        // The failed open already started the streak and said so.
        let Some(sink) = self.sink.as_mut() else {
            return;
        };

        match sink.file.write_all(line.as_bytes()) {
            Ok(()) => {
                sink.size = sink
                    .size
                    .saturating_add(u64::try_from(line.len()).unwrap_or(u64::MAX));
                self.streak.recovered();
            }
            Err(error) => {
                if self.streak.failed() {
                    report(&format!("cannot write the log: {error}"));
                }
            }
        }
    }
}

fn open_sink(path: &Path) -> io::Result<Sink> {
    use std::os::unix::fs::{OpenOptionsExt as _, PermissionsExt as _};

    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits().cast_signed())
        .open(path)?;

    // The umask or an older run may have left looser bits, and the log
    // holds paths and backtraces. Set through the open file, not the path.
    let looser = file
        .metadata()
        .is_ok_and(|meta| meta.permissions().mode() & 0o777 != 0o600);

    if looser {
        file.set_permissions(fs::Permissions::from_mode(0o600)).ok();
    }

    let size = file.metadata().map_or(0, |meta| meta.len());

    Ok(Sink { file, size })
}

/// `x.log` → `x.log.1` → … → `x.log.KEEP` (the oldest is dropped).
/// A logger has no channel for its own failures, so rename errors go to
/// stderr instead of vanishing. Missing older files are normal until the
/// history fills up.
fn rotate(path: &Path) {
    let numbered = |n: u32| {
        let mut name = path.as_os_str().to_owned();
        name.push(format!(".{n}"));
        PathBuf::from(name)
    };

    for n in (1..KEEP).rev() {
        if let Err(error) = fs::rename(numbered(n), numbered(n + 1))
            && error.kind() != io::ErrorKind::NotFound
        {
            report(&format!("cannot rotate {}: {error}", numbered(n).display()));
        }
    }

    if let Err(error) = fs::rename(path, numbered(1))
        && error.kind() != io::ErrorKind::NotFound
    {
        report(&format!("cannot rotate {}: {error}", path.display()));
    }
}

/// One line on stderr, which never touches the log file.
fn report(message: &str) {
    writeln!(io::stderr(), "garuda-toolbox: {message}").ok();
}

#[cfg(test)]
mod tests;
