//! The user's side of an AUR build: makepkg runs in paru's clone of the
//! package, and its packages go to a run folder the helper accepts.

#[cfg(test)]
mod tests;

use std::io;
use std::os::fd::OwnedFd;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rustix::fs::{AtFlags, FileType, Mode};
use rustix::io::Errno;
use toolbox_plan::{AUR_BUILT, AUR_CLONE, BuildRun, MAKEPKG_ARGS, PackageName};
use toolbox_pty::PtyCommand;

use super::review::{Unreviewed, unchanged_since_review};
use super::route::{RouteProblem, route_from_home};
use super::{entries, remove_run_folder};
use crate::secure::{self, DIR_FLAGS};

/// Run folders older than this go when the next build starts.
const KEEP_FOR: Duration = Duration::from_hours(24);

/// The helper refuses a package file that another user may write, and the
/// user's own umask may leave group write on it, so makepkg gets 022 alone.
const MAKEPKG_UMASK: Mode = Mode::from_raw_mode(0o022);

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("{0} is missing, so there is no clone to build: fetch it first")]
    NoClone(String),
    #[error("{0} is a link or no folder, and links are not followed")]
    NotAFolder(String),
    #[error(transparent)]
    Route(#[from] RouteProblem),
    #[error("{base}: {reason}")]
    Unreviewed { base: String, reason: Unreviewed },
    #[error("cannot prepare the build run: {0}")]
    Io(#[from] io::Error),
}

/// The command that builds `base` in its clone, with the packages going to
/// the run folder. The clone must hold the commit the user reviewed, with
/// no change to a tracked file. Old run folders go first, and the run folder
/// is made with mode 0700 whatever the umask.
///
/// # Errors
///
/// [`BuildError::NoClone`] or [`BuildError::NotAFolder`] when the clone is
/// not a folder of `home` reached without a link, [`BuildError::Unreviewed`]
/// when the clone differs from its review, or [`BuildError::Io`] when a run
/// folder cannot be made.
pub fn prepare(
    home: &Path,
    (makepkg, git): (&Path, &Path),
    (base, run): (&PackageName, &BuildRun),
    now: SystemTime,
    cancel: &Arc<AtomicBool>,
) -> Result<PtyCommand, BuildError> {
    let clone = Path::new(AUR_CLONE).join(base.as_str());
    let shown = format!("~/{AUR_CLONE}/{base}");

    match secure::open_dir(home, &clone) {
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(BuildError::NoClone(shown));
        }
        Err(_) => return Err(BuildError::NotAFolder(shown)),
    }

    route_from_home(home, rustix::process::geteuid().as_raw())?;

    unchanged_since_review(git, &home.join(&clone), cancel).map_err(|reason| {
        BuildError::Unreviewed {
            base: shown.clone(),
            reason,
        }
    })?;

    let built = secure::create_dirs(home, Path::new(AUR_BUILT))?;
    remove_old_runs(&built, now)?;
    run_folder(&built, run)?;

    Ok(PtyCommand::new(makepkg)
        .args(MAKEPKG_ARGS)
        .umask(MAKEPKG_UMASK)
        .env("PKGDEST", home.join(AUR_BUILT).join(run.as_str()))
        .in_folder(home.join(clone)))
}

/// Removes the run folders older than [`KEEP_FOR`]. Anything else in the
/// folder stays, and a folder that holds more than files is kept too.
fn remove_old_runs(built: &OwnedFd, now: SystemTime) -> io::Result<()> {
    for name in entries(built)? {
        let stat = match rustix::fs::statat(built, name.as_c_str(), AtFlags::SYMLINK_NOFOLLOW) {
            Ok(stat) => stat,
            Err(Errno::NOENT) => continue,
            Err(error) => return Err(error.into()),
        };

        if FileType::from_raw_mode(stat.st_mode) != FileType::Directory {
            tracing::warn!("{name:?} in the build folder is no folder, so it stays");
            continue;
        }

        if !is_older_than(stat.st_mtime, now) {
            continue;
        }

        if let Err(error) = remove_run_folder(built, name.as_c_str()) {
            tracing::warn!("an old build run stays: {error}");
        }
    }

    Ok(())
}

/// A time in the future counts as now, so it is kept.
fn is_older_than(mtime: i64, now: SystemTime) -> bool {
    let seconds = u64::try_from(mtime).unwrap_or_default();

    UNIX_EPOCH
        .checked_add(Duration::from_secs(seconds))
        .is_some_and(|modified| now.duration_since(modified).is_ok_and(|age| age > KEEP_FOR))
}

/// The run folder, made when it is missing. The mode is set after the
/// create, as the umask would take bits away from 0700 only, never add
/// any, but a mode that loses owner bits would stop makepkg.
fn run_folder(built: &OwnedFd, run: &BuildRun) -> io::Result<()> {
    match rustix::fs::mkdirat(built, run.as_str(), Mode::RWXU) {
        Ok(()) | Err(Errno::EXIST) => {}
        Err(error) => return Err(error.into()),
    }

    let folder = rustix::fs::openat(built, run.as_str(), DIR_FLAGS, Mode::empty())?;
    rustix::fs::fchmod(&folder, Mode::RWXU)?;

    Ok(())
}
