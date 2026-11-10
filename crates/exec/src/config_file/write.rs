//! Replacing a config file with the text the user edited: only while the
//! file still is the version the edit started from, with its owner and
//! mode, and with the previous version kept in the backup folder.

#[cfg(test)]
mod tests;

use std::io;
use std::path::Path;

use toolbox_plan::protocol::Outcome;
use toolbox_plan::{ConfigFilePath, ConfigText, ConfigWrite, FetchRefusal, Fingerprint, StepId};

use super::resolve::{RealPath, resolve};
use super::{ReadError, open_folder, read_text};
use crate::apply::EventSink;
use crate::builtin::FileChange;
use crate::report::{failure, note};
use crate::secure::{self, Attributes};

/// A backup may hold what the file kept from other users.
const PRIVATE_MODE: u32 = 0o600;

#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    #[error("{}", refusal_text(*.0))]
    Refused(FetchRefusal),
    /// The asked path passed the write table, but its links lead to a
    /// file outside it.
    #[error("the toolbox never writes {real}")]
    NotWritable { real: ConfigFilePath },
    #[error("the file changed since it was opened")]
    Changed,
    #[error("{0}")]
    Failed(#[source] io::Error),
}

impl From<ReadError> for WriteError {
    fn from(error: ReadError) -> Self {
        match error {
            ReadError::Refused(reason) => Self::Refused(reason),
            ReadError::Failed(error) => Self::Failed(error),
        }
    }
}

impl From<io::Error> for WriteError {
    fn from(error: io::Error) -> Self {
        Self::Failed(error)
    }
}

/// Replaces `path` below `root`, or the file its symlinks lead to, with
/// `content`. The link itself stays as it is.
///
/// # Errors
///
/// Like [`write_real`], and [`WriteError::Refused`] for a symlink that
/// leads to nothing the toolbox edits.
#[cfg(test)]
pub fn write_config_file(
    root: &Path,
    path: &ConfigFilePath,
    content: &ConfigText,
    original: Fingerprint,
) -> Result<FileChange, WriteError> {
    let real = resolve(root, path).map_err(ReadError::from)?;

    write_real(root, &real.path, content, original)
}

/// Replaces the real path `path` below `root` with `content`. No symlink
/// is followed, neither as the file nor as a folder on the way.
///
/// Atomically, with the owner and mode the file has, and with its previous
/// version kept in the backup folder (mode 0600). Only while the file
/// still has the mark `original`.
///
/// # Errors
///
/// [`WriteError::Changed`] when the file changed since it was read,
/// [`WriteError::Refused`] when it is no file the toolbox edits anymore,
/// which a symlink put in the way since the path was resolved is too,
/// [`WriteError::Failed`] when a write fails. The file is untouched then.
pub fn write_real(
    root: &Path,
    path: &ConfigFilePath,
    content: &ConfigText,
    original: Fingerprint,
) -> Result<FileChange, WriteError> {
    let (folder, name) = path.split();
    let dir = open_folder(root, folder)?;
    let (current, attributes) = read_text(&dir, name)?;

    if current.fingerprint() != original {
        return Err(WriteError::Changed);
    }

    if current == *content {
        return Ok(FileChange::Unchanged);
    }

    let backup = path.backup();
    let (backup_folder, backup_name) = backup.rsplit_once('/').unwrap_or(("", backup.as_str()));
    let backups = secure::create_dirs(root, Path::new(backup_folder))?;

    let root_owned = Attributes {
        mode: PRIVATE_MODE,
        owner: rustix::process::geteuid().as_raw(),
        group: rustix::process::getegid().as_raw(),
    };

    toolbox_fs::replace_at(
        &backups,
        backup_name,
        current.as_str().as_bytes(),
        &root_owned.replace(),
    )?
    .synced()?;
    toolbox_fs::replace_at(
        &dir,
        name,
        content.as_str().as_bytes(),
        &attributes.replace(),
    )?
    .synced()?;
    Ok(FileChange::Written)
}

/// The message names the real path, as what a link leads to is what gets written.
pub fn execute(
    root: &Path,
    step: StepId,
    write: &ConfigWrite,
    sink: &mut dyn EventSink,
) -> Outcome {
    let asked = &write.path;

    let (file, backup, result) = match resolve(root, asked) {
        Ok(real) => (
            named(&real, asked),
            real.path.backup(),
            writable(&real)
                .and_then(|()| write_real(root, &real.path, &write.content, write.original)),
        ),
        Err(error) => (
            asked.to_string(),
            asked.backup(),
            Err(WriteError::from(ReadError::from(error))),
        ),
    };

    match result {
        Ok(FileChange::Written) => note(
            step,
            sink,
            &format!("wrote {file}, the previous version is /{backup}"),
            Outcome::Succeeded,
        ),
        Ok(FileChange::Unchanged | FileChange::Removed) => note(
            step,
            sink,
            &format!("{file} is already as wanted"),
            Outcome::Skipped,
        ),
        Err(WriteError::Changed) => failure(
            step,
            sink,
            &format!(
                "{file} changed since it was opened, so it stays as it is. The edited copy is kept."
            ),
        ),
        Err(error @ WriteError::NotWritable { real: _ }) => {
            failure(step, sink, &format!("{file} cannot be replaced: {error}"))
        }
        Err(WriteError::Refused(reason)) => failure(
            step,
            sink,
            &format!("{file} cannot be replaced: {}", refusal_text(reason)),
        ),
        Err(WriteError::Failed(error)) => {
            failure(step, sink, &format!("cannot replace {file}: {error}"))
        }
    }
}

/// Decoding checked the asked path only, and a link in a writable folder
/// may lead to a file the page only shows.
fn writable(real: &RealPath) -> Result<(), WriteError> {
    if real.path.is_writable() {
        return Ok(());
    }

    Err(WriteError::NotWritable {
        real: real.path.clone(),
    })
}

fn named(real: &RealPath, asked: &ConfigFilePath) -> String {
    if real.linked {
        format!("{} (where the link {asked} leads)", real.path)
    } else {
        real.path.to_string()
    }
}

/// English, like every message of a step in the Console.
pub const fn refusal_text(reason: FetchRefusal) -> &'static str {
    match reason {
        FetchRefusal::Missing => "the file does not exist",
        FetchRefusal::Link => "a link stands in its place since it was looked at",
        FetchRefusal::Dangling => "it is a link that leads to no file",
        FetchRefusal::Outside => "it is a link to a file outside the folders the toolbox edits",
        FetchRefusal::NotRegular => "it is not a regular file",
        FetchRefusal::TooLarge => "the file is too large",
        FetchRefusal::NotText => "the file is no text",
    }
}
