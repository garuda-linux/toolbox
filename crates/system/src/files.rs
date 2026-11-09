//! Text is read lossily: one stray byte in a comment field of
//! `/etc/passwd` must not hide every account.

#[cfg(test)]
mod tests;

use std::path::Path;
use std::time::SystemTime;
use std::{fs, io};

use crate::error::{ProbeError, from_io};

/// A file the source cannot do without. Missing means unavailable.
pub fn read(path: &Path) -> Result<String, ProbeError> {
    read_lossy(path).map_err(|error| from_io(path.display(), &error))
}

/// A file whose absence is an answer of its own.
pub fn read_optional(path: &Path) -> Result<Option<String>, ProbeError> {
    match read_lossy(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(from_io(path.display(), &error)),
    }
}

pub fn modified_optional(path: &Path) -> Result<Option<SystemTime>, ProbeError> {
    match fs::metadata(path).and_then(|metadata| metadata.modified()) {
        Ok(time) => Ok(Some(time)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(from_io(path.display(), &error)),
    }
}

/// The text of `path`, lossy, with the error kind kept for callers that
/// tell a missing file from an unreadable one.
pub fn read_lossy(path: &Path) -> io::Result<String> {
    Ok(toolbox_plan::lines::lossy(fs::read(path)?))
}
