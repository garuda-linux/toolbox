//! When pacman last finished a transaction, from `/var/log/pacman.log`.

mod recent;
#[cfg(test)]
mod tests;

use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub use recent::RecentLogSource;

use super::timestamp::parse_timestamp;
use crate::error::{ProbeError, from_io};

const SYSTEM_LOG: &str = "/var/log/pacman.log";

/// The last transaction is nearly always this close to the end. The
/// log of an old system has tens of megabytes.
const TAIL_BYTES: u64 = 64 * 1024;
/// How much more is read when the end holds no transaction.
const WIDEN_BY: u64 = 4;

/// What pacman writes after every install, removal or upgrade that went
/// through. A failed transaction writes `transaction failed` instead.
const COMPLETED: &str = "[ALPM] transaction completed";

pub trait UpdateLogSource: Send + Sync {
    /// The time of the last completed transaction, `None` when the log
    /// holds none.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] when the log is missing or not readable
    /// for the user, [`ProbeError::Failed`] when reading it breaks off.
    fn last_update(&self) -> Result<Option<SystemTime>, ProbeError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PacmanLog {
    path: PathBuf,
}

impl PacmanLog {
    #[must_use = "the adapter does nothing until asked"]
    pub fn system() -> Self {
        Self::new(SYSTEM_LOG)
    }

    /// Another log file, for tests.
    #[must_use = "the adapter does nothing until asked"]
    pub(crate) fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl UpdateLogSource for PacmanLog {
    /// The log is world readable on a stock system, so no password is
    /// asked. A closed `/var/log` reads as not available.
    fn last_update(&self) -> Result<Option<SystemTime>, ProbeError> {
        let mut file = open_log(&self.path)?;

        last_in_tail(&mut file, TAIL_BYTES).map_err(|error| unreadable(&self.path, error))
    }
}

/// Opens the log as a file. A folder opens fine, and what reading it gives
/// depends on the file system (tmpfs seeks to 0 and reads nothing, a "no
/// update").
fn open_log(path: &Path) -> Result<File, ProbeError> {
    let file = File::open(path).map_err(|error| unreadable(path, error))?;
    let kind = file.metadata().map_err(|error| unreadable(path, error))?;

    if !kind.is_file() {
        return Err(ProbeError::Failed(format!("{} is no file", path.display())));
    }

    Ok(file)
}

/// The reader of the whole lines in the last bytes of a log `length` bytes
/// long, from the line that holds `start` on.
fn whole_lines<L>(log: &mut L, length: u64, start: u64) -> io::Result<BufReader<io::Take<&mut L>>>
where
    L: Read + Seek,
{
    // One byte early: the line that is skipped is then only a line
    // break when the window starts at a line.
    log.seek(SeekFrom::Start(start.saturating_sub(1)))?;

    let mut tail = BufReader::new(log.by_ref().take(length - start.saturating_sub(1)));

    if start > 0 {
        tail.skip_until(b'\n')?;
    }

    Ok(tail)
}

fn last_in_tail<L>(log: &mut L, window: u64) -> io::Result<Option<SystemTime>>
where
    L: Read + Seek,
{
    let length = log.seek(SeekFrom::End(0))?;
    let mut window = window.max(1);

    loop {
        let start = length.saturating_sub(window);
        let last = last_completed(whole_lines(log, length, start)?)?;

        if last.is_some() || start == 0 {
            return Ok(last);
        }

        window = window.saturating_mul(WIDEN_BY);
    }
}

/// Lines are read as lossy UTF-8: scriptlets print anything, and one bad byte must
/// not hide the lines after it. Lines whose timestamp cannot be read are
/// skipped.
///
/// # Errors
///
/// When reading `log` fails.
fn last_completed<R>(log: R) -> io::Result<Option<SystemTime>>
where
    R: BufRead,
{
    let mut last = None;

    for line in log.split(b'\n') {
        let line = line?;
        let text = String::from_utf8_lossy(&line);

        if let Some(time) = completed_at(&text) {
            last = Some(time);
        }
    }

    Ok(last)
}

/// `[timestamp] [ALPM] transaction completed`.
fn completed_at(line: &str) -> Option<SystemTime> {
    let (stamp, message) = line.strip_prefix('[')?.split_once(']')?;

    (message.trim() == COMPLETED)
        .then(|| parse_timestamp(stamp))
        .flatten()
}

fn unreadable(path: &Path, error: io::Error) -> ProbeError {
    if error.kind() == io::ErrorKind::PermissionDenied {
        tracing::info!("{}: {error}", path.display());
        ProbeError::Unavailable
    } else {
        from_io(path.display(), error)
    }
}
