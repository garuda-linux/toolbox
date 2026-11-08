//! Replacing a config file with the text the user edited: only while the
//! file still is the version the edit started from, with its owner and
//! mode, and with the previous version kept in the backup folder.

#[cfg(test)]
mod tests;

use std::io::Write as _;
use std::os::unix::fs::OpenOptionsExt as _;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use std::{fmt, io};

use toolbox_plan::protocol::Outcome;
use toolbox_plan::{ConfigFilePath, ConfigText, ConfigWrite, FetchRefusal, Fingerprint, StepId};

use super::resolve::{RealPath, resolve};
use super::{ReadError, open_folder, read_text};
use crate::apply::EventSink;
use crate::builtin::FileChange;
use crate::capture::{CaptureError, MAX_ERROR_BYTES, Policy, capture};
use crate::pty_runner::Environment;
use crate::report::{failure, note};
use crate::secure::{self, Attributes};

/// A backup or a sudoers candidate may hold what the file kept from other
/// users.
const PRIVATE_MODE: u32 = 0o600;
const SHOWN_VISUDO_LINES: usize = 3;
/// The one config file with a checker of its own: a typo in it locks
/// sudo for every user.
const SUDOERS: &str = "/etc/sudoers";
const VISUDO: &str = "/usr/bin/visudo";
/// visudo reads one small file. One that hangs must not hold the helper.
const VISUDO_DEADLINE: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub enum WriteError {
    Refused(FetchRefusal),
    Changed,
    Invalid(String),
    Failed(io::Error),
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused(reason) => f.write_str(refusal_text(*reason)),
            Self::Changed => f.write_str("the file changed since it was opened"),
            Self::Invalid(reason) => f.write_str(reason),
            Self::Failed(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for WriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Failed(error) => Some(error),
            Self::Refused(_) | Self::Changed | Self::Invalid(_) => None,
        }
    }
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

    if needs_check(path) {
        check_sudoers(root, path, content)?;
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

fn needs_check(path: &ConfigFilePath) -> bool {
    path.as_str() == SUDOERS
}

/// Runs `visudo -c` over the new text of `/etc/sudoers`: a typo there
/// locks sudo for every user, and the backup is no help until someone
/// notices.
fn check_sudoers(
    root: &Path,
    path: &ConfigFilePath,
    content: &ConfigText,
) -> Result<(), WriteError> {
    let (folder, name) = path.split();
    let candidate = root
        .join(folder)
        .join(format!(".{name}.toolbox-check-{}", std::process::id()));

    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(PRIVATE_MODE)
        .open(&candidate)
        .and_then(|mut file| file.write_all(content.as_str().as_bytes()))
        .map_err(WriteError::Failed)?;

    let mut visudo = Command::new(crate::located::fixed(VISUDO));

    visudo.args(["-c", "-f"]).arg(&candidate);

    let checked = capture(
        &mut visudo,
        Environment::Clean,
        &[],
        &Policy::tail(MAX_ERROR_BYTES).within(VISUDO_DEADLINE),
        &Arc::default(),
    );

    std::fs::remove_file(&candidate).ok();

    let captured = checked.map_err(unchecked)?;

    if captured.code == Some(0_i32) {
        return Ok(());
    }

    let reason: String = captured
        .errors
        .lines()
        .take(SHOWN_VISUDO_LINES)
        .collect::<Vec<_>>()
        .join("\n");

    Err(WriteError::Invalid(format!(
        "visudo refuses the file: {reason}"
    )))
}

/// Without a verdict the file stays as it is.
fn unchecked(error: CaptureError) -> WriteError {
    match error {
        CaptureError::Missing => {
            WriteError::Invalid("visudo is missing, so the file cannot be checked".to_owned())
        }
        CaptureError::TimedOut => WriteError::Invalid(format!(
            "visudo gave no answer within {} seconds, so the file is not written",
            VISUDO_DEADLINE.as_secs()
        )),
        CaptureError::Failed(_) | CaptureError::Cancelled | CaptureError::TooLarge => {
            WriteError::Failed(error.into())
        }
    }
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
            write_real(root, &real.path, &write.content, write.original),
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
        Err(WriteError::Invalid(reason)) => failure(
            step,
            sink,
            &format!("{file} stays as it is: {reason}. The edited copy is kept."),
        ),
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
