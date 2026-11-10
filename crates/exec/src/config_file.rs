//! Reading and replacing the config files the Config Files page edits.
//! The helper does both as root, the app reads with the same code as the
//! user, so both refuse the same files. Symlinks are followed once, to the
//! real path, which must again be a file the toolbox edits (`resolve`).
//! From there every folder is opened from the root without following a
//! symlink, only regular text files of a sane size pass, and a file is
//! replaced only while it still is the version the edit started from.

mod resolve;
#[cfg(test)]
mod tests;
mod write;

use std::os::fd::OwnedFd;
use std::path::Path;
use std::{fmt, io};

use resolve::{Unresolved, resolve};
use rustix::fs::{AtFlags, FileType};
use toolbox_plan::{ConfigFilePath, ConfigText, FetchRefusal, InvalidConfigText, MAX_CONFIG_BYTES};
use write::refusal_text;
pub use write::{WriteError, execute};

use crate::secure::{self, Attributes, Unreadable};

/// More files than any config folder holds. A longer listing is cut.
pub const MAX_FOLDER_FILES: usize = 500;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Found {
    File {
        link: Option<ConfigFilePath>,
    },
    /// Sorted names of its regular files.
    Folder {
        files: Vec<String>,
        link: Option<ConfigFilePath>,
    },
    /// A symlink to nothing the toolbox edits: where it leads as far as
    /// it could be followed, and why that is not edited.
    Link {
        target: String,
        reason: FetchRefusal,
    },
    Missing,
    /// A folder on the way may not be entered by this user.
    Hidden,
    /// A FIFO, socket or device.
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigRead {
    pub path: ConfigFilePath,
    pub content: ConfigText,
}

#[derive(Debug)]
pub enum ReadError {
    Refused(FetchRefusal),
    /// Reading failed, for the app usually because only root may.
    Failed(io::Error),
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused(reason) => f.write_str(refusal_text(*reason)),
            Self::Failed(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Failed(error) => Some(error),
            Self::Refused(_) => None,
        }
    }
}

impl From<Unreadable> for ReadError {
    fn from(error: Unreadable) -> Self {
        match error {
            Unreadable::Missing => Self::Refused(FetchRefusal::Missing),
            Unreadable::Link => Self::Refused(FetchRefusal::Link),
            Unreadable::NotRegular => Self::Refused(FetchRefusal::NotRegular),
            Unreadable::TooLarge => Self::Refused(FetchRefusal::TooLarge),
            Unreadable::Failed(error) => Self::Failed(error),
        }
    }
}

impl From<Unresolved> for ReadError {
    fn from(error: Unresolved) -> Self {
        match error {
            Unresolved::Missing => Self::Refused(FetchRefusal::Missing),
            Unresolved::Refused { reason, target: _ } => Self::Refused(reason),
            Unresolved::Failed(error) => Self::Failed(error),
        }
    }
}

pub fn inspect(root: &Path, path: &ConfigFilePath) -> Found {
    let real = match resolve(root, path) {
        Ok(real) => real,
        Err(Unresolved::Refused { reason, target }) => return Found::Link { target, reason },
        Err(Unresolved::Failed(error)) if error.kind() == io::ErrorKind::PermissionDenied => {
            return Found::Hidden;
        }
        Err(Unresolved::Missing | Unresolved::Failed(_)) => return Found::Missing,
    };

    let link = real.linked.then(|| real.path.clone());

    match look(root, &real.path) {
        Found::File { link: _ } => Found::File { link },
        Found::Folder { files, link: _ } => Found::Folder { files, link },
        Found::Other => link.map_or(Found::Other, |target| Found::Link {
            target: target.to_string(),
            reason: FetchRefusal::NotRegular,
        }),
        found @ (Found::Link {
            target: _,
            reason: _,
        }
        | Found::Missing
        | Found::Hidden) => found,
    }
}

/// What the real path `path` is, without following a symlink. One that
/// appeared since the path was resolved counts as missing: the next look
/// follows it.
fn look(root: &Path, path: &ConfigFilePath) -> Found {
    let (folder, name) = path.split();
    let dir = match open_folder(root, folder) {
        Ok(dir) => dir,
        Err(ReadError::Failed(error)) if error.kind() == io::ErrorKind::PermissionDenied => {
            return Found::Hidden;
        }
        Err(ReadError::Refused(_) | ReadError::Failed(_)) => return Found::Missing,
    };

    let Ok(stat) = rustix::fs::statat(&dir, name, AtFlags::SYMLINK_NOFOLLOW) else {
        return Found::Missing;
    };

    match FileType::from_raw_mode(stat.st_mode) {
        FileType::RegularFile => Found::File { link: None },
        FileType::Symlink => Found::Missing,
        FileType::Directory => match open_folder(root, path.relative())
            .map_err(|_refused| ())
            .and_then(|opened| secure::regular_files(&opened).map_err(|_unlisted| ()))
        {
            Ok(mut files) => {
                files.truncate(MAX_FOLDER_FILES);
                Found::Folder { files, link: None }
            }
            Err(()) => Found::Hidden,
        },
        FileType::Fifo
        | FileType::Socket
        | FileType::CharacterDevice
        | FileType::BlockDevice
        | FileType::Unknown => Found::Other,
    }
}

/// # Errors
///
/// [`ReadError::Refused`] for a missing file, a symlink that leads to
/// nothing the toolbox edits, and a file that is not regular, too large or
/// no text. [`ReadError::Failed`] when the system refuses the read.
pub fn read_config_file(root: &Path, path: &ConfigFilePath) -> Result<ConfigRead, ReadError> {
    let real = resolve(root, path)?;
    let content = read_real(root, &real.path)?;

    Ok(ConfigRead {
        path: real.path,
        content,
    })
}

/// No symlink is followed, neither as the file nor as a folder on the way.
///
/// # Errors
///
/// Like [`read_config_file`], and [`FetchRefusal::Link`] for a symlink
/// that stands in the way since the path was resolved.
pub fn read_real(root: &Path, path: &ConfigFilePath) -> Result<ConfigText, ReadError> {
    let (folder, name) = path.split();
    let dir = open_folder(root, folder)?;

    read_text(&dir, name).map(|(text, _)| text)
}

/// A missing folder, or a symlink where one should be, is an answer.
fn open_folder(root: &Path, folder: &str) -> Result<OwnedFd, ReadError> {
    secure::open_dir(root, Path::new(folder)).map_err(|error| {
        error
            .raw_os_error()
            .map(rustix::io::Errno::from_raw_os_error)
            .map_or_else(
                || ReadError::Failed(error),
                |errno| Unreadable::from(errno).into(),
            )
    })
}

fn read_text(dir: &OwnedFd, name: &str) -> Result<(ConfigText, Attributes), ReadError> {
    let file = secure::read_regular(dir, name, MAX_CONFIG_BYTES)?;
    let text = ConfigText::from_bytes(file.content).map_err(|error| {
        ReadError::Refused(match error {
            InvalidConfigText::TooLarge { max: _ } => FetchRefusal::TooLarge,
            InvalidConfigText::NotText => FetchRefusal::NotText,
        })
    })?;

    Ok((text, file.attributes))
}
