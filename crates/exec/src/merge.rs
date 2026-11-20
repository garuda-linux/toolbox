//! Resolving a copy as root. Trusted only when pacman's database lists
//! its file as a backup file. No symlinks, and both files unchanged.

#[cfg(test)]
mod tests;

use std::io;
use std::path::Path;

use rustix::fs::RenameFlags;
use toolbox_plan::procedure::{Command, Program};
use toolbox_plan::protocol::Outcome;
use toolbox_plan::{
    ConfigFilePath, ConfigText, FetchRefusal, MergeCopy, MergeResolve, PACKAGE_FILES, PACMAN_LOCAL,
    Resolution, SNAPPER_ROOT_CONFIG, StepId, lists_backup,
};

use crate::apply::EventSink;
use crate::config_file::{ReadError, WriteError, open_folder, read_text, write_real};
use crate::report::{failure, note};
use crate::secure;

/// Far above the longest list a package has, `linux-firmware`'s.
const MAX_FILES_LIST_BYTES: usize = 64 << 20;

const COPY_SUFFIXES: [&str; 3] = [".pacnew", ".pacsave", ".pacorig"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resolved {
    Removed,
    Moved,
    Written,
}

#[derive(Debug, thiserror::Error)]
pub enum MergeError {
    #[error("pacman lists no backup file of that name")]
    NoBackup,
    #[error(transparent)]
    Write(#[from] WriteError),
    #[error("the syntax check refused the new text")]
    Refused(Outcome),
}

impl From<io::Error> for MergeError {
    fn from(error: io::Error) -> Self {
        Self::Write(error.into())
    }
}

impl From<ReadError> for MergeError {
    fn from(error: ReadError) -> Self {
        Self::Write(error.into())
    }
}

/// Whether a package's `files` list below `root` names `original` as a
/// backup file. A package folder that cannot be read counts as not naming it.
///
/// # Errors
///
/// When the database folder cannot be opened or listed.
pub fn is_backup(root: &Path, original: &ConfigFilePath) -> io::Result<bool> {
    let local = secure::open_dir(root, Path::new(PACMAN_LOCAL))?;

    for entry in rustix::fs::Dir::read_from(&local)? {
        let entry = entry?;
        let Ok(name) = entry.file_name().to_str() else {
            continue;
        };

        if name == "." || name == ".." {
            continue;
        }

        let Ok(package) = secure::open_dir(root, &Path::new(PACMAN_LOCAL).join(name)) else {
            continue;
        };

        let Ok(files) = secure::read_regular(&package, PACKAGE_FILES, MAX_FILES_LIST_BYTES) else {
            continue;
        };

        if lists_backup(&String::from_utf8_lossy(&files.content), original) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Whether `fetch` may hand out `path` although no catalog entry names
/// it: a copy of a backup file, or a backup file with a copy beside it.
pub fn is_fetchable(root: &Path, path: &ConfigFilePath) -> bool {
    let original = match MergeCopy::new(path.as_str()) {
        Ok(copy) => copy.original().clone(),
        Err(_no_copy) if has_copy(root, path) => path.clone(),
        Err(_no_copy) => return false,
    };

    is_backup(root, &original).unwrap_or(false)
}

/// Where snapper is installed and has a configuration of `/` below `root`.
pub fn snapshot(root: &Path) -> Option<Command> {
    let installed = Program::Snapper.below(root).is_file();
    let configured = root.join(SNAPPER_ROOT_CONFIG).is_file();

    (installed && configured).then(toolbox_plan::merge_snapshot)
}

/// # Errors
///
/// [`MergeError`] when the resolve is refused, leaving both files untouched.
pub fn resolve(
    root: &Path,
    merge: &MergeResolve,
    check: impl FnOnce(&Command, &ConfigText) -> Outcome,
) -> Result<Resolved, MergeError> {
    if !is_backup(root, merge.copy.original())? {
        return Err(MergeError::NoBackup);
    }

    resolve_pair(root, merge, check)
}

pub fn execute(
    root: &Path,
    step: StepId,
    merge: &MergeResolve,
    sink: &mut dyn EventSink,
    check: impl FnOnce(&Command, &ConfigText, &mut dyn EventSink) -> Outcome,
) -> Outcome {
    let copy = merge.copy.path();
    let original = merge.copy.original();
    let resolved = resolve(root, merge, |command, text| {
        check(command, text, &mut *sink)
    });

    match resolved {
        Ok(Resolved::Removed) => note(step, sink, &format!("removed {copy}"), Outcome::Succeeded),
        Ok(Resolved::Moved) => note(
            step,
            sink,
            &format!("moved {copy} to {original}"),
            Outcome::Succeeded,
        ),
        Ok(Resolved::Written) => note(
            step,
            sink,
            &format!(
                "wrote {original}, the previous version is /{}, and removed {copy}",
                original.backup()
            ),
            Outcome::Succeeded,
        ),
        Err(MergeError::NoBackup) => failure(
            step,
            sink,
            &format!("pacman lists no backup file {original}, so {copy} stays"),
        ),
        Err(MergeError::Write(WriteError::Changed)) => failure(
            step,
            sink,
            &format!(
                "{copy} or {original} changed since they were compared, so both stay as they are"
            ),
        ),
        Err(MergeError::Refused(outcome)) => note(
            step,
            sink,
            &format!("visudo refused the new text of {original}, so both files stay as they are"),
            outcome,
        ),
        Err(MergeError::Write(WriteError::Failed(error))) => {
            failure(step, sink, &format!("cannot resolve {copy}: {error}"))
        }
        Err(
            error @ MergeError::Write(WriteError::Refused(_) | WriteError::NotWritable { real: _ }),
        ) => failure(step, sink, &format!("{copy} cannot be resolved: {error}")),
    }
}

fn resolve_pair(
    root: &Path,
    merge: &MergeResolve,
    check: impl FnOnce(&Command, &ConfigText) -> Outcome,
) -> Result<Resolved, MergeError> {
    let (folder, copy_name) = merge.copy.path().split();
    let (_, name) = merge.copy.original().split();
    let dir = open_folder(root, folder)?;
    let (copied, _) = read_text(&dir, copy_name)?;
    let current = match read_text(&dir, name) {
        Ok((current, _)) => Some(current),
        Err(ReadError::Refused(FetchRefusal::Missing)) => None,
        Err(error) => return Err(error.into()),
    };

    if copied.fingerprint() != merge.choice.copy
        || current.as_ref().map(ConfigText::fingerprint) != merge.choice.current
    {
        return Err(WriteError::Changed.into());
    }

    let (wanted, mark) = match (&merge.choice.resolution, merge.choice.current) {
        (Resolution::KeepCurrent, _) => {
            secure::remove_file(&dir, copy_name)?;
            return Ok(Resolved::Removed);
        }
        (Resolution::Merged { text: _ }, None) => return Err(WriteError::Changed.into()),
        (Resolution::TakeCopy, mark) => (copied, mark),
        (Resolution::Merged { text }, Some(mark)) => (text.clone(), Some(mark)),
    };

    if let Some(command) = merge.syntax_check() {
        let outcome = check(&command, &wanted);

        if outcome != Outcome::Succeeded {
            return Err(MergeError::Refused(outcome));
        }
    }

    let Some(mark) = mark else {
        rustix::fs::renameat_with(&dir, copy_name, &dir, name, RenameFlags::NOREPLACE)
            .map_err(io::Error::from)?;
        rustix::fs::fsync(&dir).map_err(io::Error::from)?;
        return Ok(Resolved::Moved);
    };

    write_real(root, merge.copy.original(), &wanted, mark)?;
    secure::remove_file(&dir, copy_name)?;
    Ok(Resolved::Written)
}

fn has_copy(root: &Path, path: &ConfigFilePath) -> bool {
    let (folder, name) = path.split();

    open_folder(root, folder).is_ok_and(|dir| {
        COPY_SUFFIXES
            .iter()
            .any(|suffix| secure::is_regular(&dir, &format!("{name}{suffix}")))
    })
}
