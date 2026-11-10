#[cfg(test)]
mod tests;

use std::io::{self, Read, Seek, SeekFrom};

use super::{PacmanLog, open_log, unreadable, whole_lines};
use crate::error::ProbeError;

/// The log of an old system has tens of megabytes. The history of one
/// package shows what fits in this many of its newest.
const RECENT_BYTES: u64 = 4 * 1024 * 1024;

pub trait RecentLogSource: Send + Sync {
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] when the log is missing or not readable
    /// for the user, [`ProbeError::Failed`] when reading it breaks off.
    fn recent(&self) -> Result<String, ProbeError>;
}

impl RecentLogSource for PacmanLog {
    fn recent(&self) -> Result<String, ProbeError> {
        let mut file = open_log(&self.path)?;

        tail_text(&mut file, RECENT_BYTES).map_err(|error| unreadable(&self.path, error))
    }
}

/// The last `window` bytes of `log` from the first whole line on, as
/// lossy UTF-8: scriptlets print anything.
pub(super) fn tail_text<L>(log: &mut L, window: u64) -> io::Result<String>
where
    L: Read + Seek,
{
    let length = log.seek(SeekFrom::End(0))?;
    let start = length.saturating_sub(window);

    let mut bytes = Vec::new();
    whole_lines(log, length, start)?.read_to_end(&mut bytes)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}
