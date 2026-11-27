//! The folders on the way from a caller's home to the folder builds are
//! staged in. The helper and the user-side build both walk it, so they refuse
//! the same setups.

use std::os::fd::OwnedFd;
use std::path::{Path, PathBuf};
use std::{fs, io};

use rustix::fs::Mode;
use rustix::io::Errno;
use toolbox_plan::lines::lossy;
use toolbox_plan::procedure::PASSWD;
use toolbox_plan::{AUR_BUILT, passwd_entries};

use super::StageError;
use crate::secure::{self, DIR_FLAGS};

const GROUP_OR_OTHERS_WRITE: u32 = 0o022;

/// The folders of the route that exist. `missing` names the ones below
/// `folder` that do not exist yet, which the user-side build creates.
#[derive(Debug)]
pub struct Route {
    pub home: PathBuf,
    pub folder: OwnedFd,
    pub missing: Vec<&'static str>,
}

#[derive(Debug, thiserror::Error)]
pub enum RouteProblem {
    #[error("user id {0} has no account in /etc/passwd")]
    NoAccount(u32),
    #[error(transparent)]
    Passwd(io::Error),
    /// The home is missing, a link or no folder.
    #[error(transparent)]
    Home(io::Error),
    /// `.cache` is missing, a link or no folder.
    #[error(transparent)]
    Cache(io::Error),
    /// `garuda-toolbox` or `aur-built` is a link or no folder, or a folder
    /// of the route cannot be examined.
    #[error("{name}: {error}")]
    Folder { name: String, error: io::Error },
    #[error("{0} does not belong to the user who asked")]
    Foreign(String),
    #[error("{0} may be written by other users")]
    Shared(String),
}

impl From<RouteProblem> for StageError {
    fn from(problem: RouteProblem) -> Self {
        match problem {
            RouteProblem::NoAccount(uid) => Self::NoAccount(uid),
            RouteProblem::Foreign(name) => Self::Foreign(name),
            RouteProblem::Shared(name) => Self::Shared(name),
            RouteProblem::Passwd(error)
            | RouteProblem::Home(error)
            | RouteProblem::Cache(error)
            | RouteProblem::Folder { error, name: _ } => Self::Io(error),
        }
    }
}

/// The route the helper takes for `uid`, with the home from `/etc/passwd`
/// below `root`.
///
/// # Errors
///
/// [`RouteProblem`] for the first setup the helper refuses on the way.
pub fn route(root: &Path, uid: u32) -> Result<Route, RouteProblem> {
    let home = home_entry(root, uid)?;

    walk(root, &home, &home, uid)
}

/// The route of the user-side build, from the home folder it builds in.
///
/// # Errors
///
/// As [`route`].
pub fn route_from_home(home: &Path, uid: u32) -> Result<Route, RouteProblem> {
    walk(home, Path::new(""), home, uid)
}

/// The home `/etc/passwd` below `root` gives `uid`, relative to `root`.
///
/// # Errors
///
/// [`RouteProblem::NoAccount`] when no entry has `uid`, or
/// [`RouteProblem::Passwd`] when the file cannot be read.
pub fn home_entry(root: &Path, uid: u32) -> Result<PathBuf, RouteProblem> {
    let passwd = lossy(fs::read(root.join(PASSWD)).map_err(RouteProblem::Passwd)?);

    passwd_entries(&passwd)
        .find(|entry| entry.uid == uid)
        .map(|entry| PathBuf::from(entry.home.trim_start_matches('/')))
        .ok_or(RouteProblem::NoAccount(uid))
}

/// Opens `rel` below `root` without following a link, then each folder of
/// [`AUR_BUILT`] that exists. The first missing folder ends the walk, and
/// the folders below it are listed as missing. `shown` names the home in
/// the problems.
pub(super) fn walk(root: &Path, rel: &Path, shown: &Path, uid: u32) -> Result<Route, RouteProblem> {
    let mut folder = secure::open_dir(root, rel).map_err(RouteProblem::Home)?;
    owned_and_closed(&folder, uid, &shown.display().to_string())?;

    let mut missing = Vec::new();
    for (index, part) in AUR_BUILT.split('/').enumerate() {
        if !missing.is_empty() {
            missing.push(part);
            continue;
        }

        match rustix::fs::openat(&folder, part, DIR_FLAGS, Mode::empty()) {
            Ok(next) => {
                owned_and_closed(&next, uid, part)?;
                folder = next;
            }
            Err(Errno::NOENT) if index > 0 => missing.push(part),
            Err(error) if index == 0 => return Err(RouteProblem::Cache(error.into())),
            Err(error) => {
                return Err(RouteProblem::Folder {
                    name: part.to_owned(),
                    error: error.into(),
                });
            }
        }
    }

    Ok(Route {
        home: shown.to_path_buf(),
        folder,
        missing,
    })
}

pub(super) fn owned_and_closed(fd: &OwnedFd, uid: u32, what: &str) -> Result<(), RouteProblem> {
    let stat = rustix::fs::fstat(fd).map_err(|error| RouteProblem::Folder {
        name: what.to_owned(),
        error: error.into(),
    })?;

    if stat.st_uid != uid {
        return Err(RouteProblem::Foreign(what.to_owned()));
    }

    if stat.st_mode & GROUP_OR_OTHERS_WRITE != 0 {
        return Err(RouteProblem::Shared(what.to_owned()));
    }

    Ok(())
}
