//! What makepkg built as the user, made ready for `pacman -U` as root. The
//! run's folder is reached from the user's home without following a link,
//! every folder on the way and every file must be the user's own and
//! closed to other users, and the files are copied into a folder of root's
//! before pacman reads them, so nothing can be swapped after the check.

#[cfg(test)]
mod tests;

use std::ffi::{CStr, CString};
use std::fs::{self, File};
use std::io;
use std::os::fd::OwnedFd;
use std::path::{Path, PathBuf};

use rustix::fs::{AtFlags, FileType, FlockOperation, Mode, OFlags};
use rustix::io::Errno;
use toolbox_plan::lines::lossy;
use toolbox_plan::procedure::{Command, PASSWD, Program};
use toolbox_plan::protocol::Outcome;
use toolbox_plan::{
    AUR_BUILT, AUR_STORE, BuildRun, Caller, InstallReason, PackageName, TargetList, passwd_entries,
};

use crate::secure::{self, DIR_FLAGS, READ_FLAGS};

/// Without `NONBLOCK` a fifo in the place of a package would hold the
/// helper at the open.
const PACKAGE_READ: OFlags = READ_FLAGS.union(OFlags::NONBLOCK);

const CREATE_FLAGS: OFlags = OFlags::WRONLY
    .union(OFlags::CREATE)
    .union(OFlags::EXCL)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::CLOEXEC);

const GROUP_OR_OTHERS_WRITE: u32 = 0o022;

const PACKAGE_MODE: Mode = Mode::from_raw_mode(0o644);

/// What follows a package's arch, before its compression.
const PACKAGE_SUFFIX: &str = ".pkg.tar";

/// The endings `PKGEXT` may give a package. A signature or a half written
/// file beside it has another ending and is no package.
const COMPRESSIONS: [&str; 9] = [
    "", ".zst", ".xz", ".gz", ".bz2", ".lz4", ".lzo", ".lz", ".Z",
];

/// [`toolbox_plan::procedure::Builtin::InstallBuilt`] without its step.
#[derive(Clone, Copy, Debug)]
pub struct Install<'step> {
    pub run: &'step BuildRun,
    pub packages: &'step TargetList<PackageName>,
    pub reason: InstallReason,
}

#[derive(Debug, thiserror::Error)]
pub enum StageError {
    #[error("{0} does not belong to the user who asked")]
    Foreign(String),
    #[error("{0} may be written by other users")]
    Shared(String),
    #[error("{0} is no regular file")]
    NotRegular(String),
    #[error("another install of built packages is using /{AUR_STORE}")]
    Busy,
    #[error("{0} in /{AUR_STORE} is no folder of a build run")]
    NoRunFolder(String),
    #[error("no package file was built for {0}")]
    NotBuilt(PackageName),
    #[error("more than one package file was built for {0}")]
    Ambiguous(PackageName),
    #[error("user id {0} has no account in /etc/passwd")]
    NoAccount(u32),
    /// The helper learns the caller from its stdin socket. Started any other
    /// way it does not know whose home to read.
    #[error("the helper does not know who asked, so it reads no built package")]
    NoCaller,
    #[error(transparent)]
    Io(#[from] io::Error),
}

impl From<Errno> for StageError {
    fn from(error: Errno) -> Self {
        Self::Io(error.into())
    }
}

/// The package a file of makepkg holds, from `<name>-<pkgver>-<pkgrel>-<arch>.pkg.tar`
/// and one of [`COMPRESSIONS`].
/// pkgver, pkgrel and arch hold no `-`, so the name is all before them.
pub fn package_of(file: &str) -> Option<&str> {
    let stem = COMPRESSIONS
        .iter()
        .find_map(|compression| file.strip_suffix(compression)?.strip_suffix(PACKAGE_SUFFIX))?;

    let mut fields = stem.rsplitn(4, '-');
    let (_arch, _rel, _version) = (fields.next()?, fields.next()?, fields.next()?);

    fields.next().filter(|name| !name.is_empty())
}

/// Stages what `caller` built, hands the copies to `pacman`, and removes
/// them again whatever pacman did.
///
/// # Errors
///
/// [`StageError`] when there is no caller or the files cannot be staged.
pub fn install(
    root: &Path,
    caller: Option<Caller>,
    built: Install<'_>,
    pacman: impl FnOnce(&Command) -> Outcome,
) -> Result<Outcome, StageError> {
    let uid = caller.ok_or(StageError::NoCaller)?.uid();
    let home = home_of(root, uid)?;
    let _held = hold_store(root)?;
    let copies = stage(root, uid, &home, built.run, built.packages)?;

    let command = Command::new(Program::Pacman)
        .arg("-U")
        .args(built.reason.pacman_flag())
        .args(["--noconfirm", "--"])
        .args(copies.iter().map(|copy| copy.display().to_string()));

    let outcome = pacman(&command);

    if let Err(error) = unstage(root, built.run) {
        tracing::warn!("the copies of build run {} stay: {error}", built.run);
    }

    Ok(outcome)
}

/// The store, locked until the descriptor closes. Every stage empties the
/// whole store, so a second helper would take the first one's copies away
/// before pacman read them.
fn hold_store(root: &Path) -> Result<OwnedFd, StageError> {
    let store = secure::create_dirs(root, Path::new(AUR_STORE))?;

    match rustix::fs::flock(&store, FlockOperation::NonBlockingLockExclusive) {
        Ok(()) => Ok(store),
        Err(Errno::WOULDBLOCK) => Err(StageError::Busy),
        Err(error) => Err(error.into()),
    }
}

/// The home folder of `uid` as `/etc/passwd` below `root` names it,
/// relative to `root`.
///
/// # Errors
///
/// [`StageError::NoAccount`] when no entry has `uid`, or the read error.
pub fn home_of(root: &Path, uid: u32) -> Result<PathBuf, StageError> {
    let passwd = lossy(fs::read(root.join(PASSWD))?);

    passwd_entries(&passwd)
        .find(|entry| entry.uid == uid)
        .map(|entry| PathBuf::from(entry.home.trim_start_matches('/')))
        .ok_or(StageError::NoAccount(uid))
}

/// Copies the one file `run` built for each of `packages` from the run's
/// folder below `home` (relative to `root`) into root's store, and returns
/// the copies' paths as seen from `root`. The store holds no copy of an
/// earlier run afterwards.
///
/// # Errors
///
/// [`StageError`] when a folder or file is not `uid`'s own or others may
/// write it, a link is on the way, or a package has no file or more than one.
pub fn stage(
    root: &Path,
    uid: u32,
    home: &Path,
    run: &BuildRun,
    packages: &TargetList<PackageName>,
) -> Result<Vec<PathBuf>, StageError> {
    let built = built_folder(root, uid, home, run)?;
    let files = secure::regular_files(&built)?;
    let chosen = packages
        .iter()
        .map(|package| only_file(&files, package))
        .collect::<Result<Vec<&String>, StageError>>()?;

    empty_store(root)?;
    let relative = Path::new(AUR_STORE).join(run.as_str());
    let store = secure::create_dirs(root, &relative)?;

    chosen
        .into_iter()
        .map(|name| {
            copy(&built, &store, name, uid)?;
            Ok(Path::new("/").join(&relative).join(name))
        })
        .collect()
}

/// Removes the copies of `run` and their folder. Nothing to remove is fine.
///
/// # Errors
///
/// When the store or a copy cannot be removed.
pub fn unstage(root: &Path, run: &BuildRun) -> Result<(), StageError> {
    let Some(store) = open_store(root)? else {
        return Ok(());
    };

    let name = CString::new(run.as_str()).map_err(io::Error::other)?;
    remove_run_folder(&store, &name)
}

/// Removes every run folder of the store, also those of other runs a crash
/// left behind.
///
/// # Errors
///
/// [`StageError`] when the store holds anything but folders of regular
/// files, or something cannot be removed.
pub fn empty_store(root: &Path) -> Result<(), StageError> {
    let Some(store) = open_store(root)? else {
        return Ok(());
    };

    for name in entries(&store)? {
        remove_run_folder(&store, &name)?;
    }

    Ok(())
}

fn open_store(root: &Path) -> io::Result<Option<OwnedFd>> {
    match secure::open_dir(root, Path::new(AUR_STORE)) {
        Ok(store) => Ok(Some(store)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// The names in `dir` but `.` and `..`, as bytes: a name that is no UTF-8
/// is removed or refused too.
fn entries(dir: &OwnedFd) -> io::Result<Vec<CString>> {
    let mut names = Vec::new();

    for entry in rustix::fs::Dir::read_from(dir)? {
        let name = entry?.file_name().to_owned();

        if name.as_bytes() != b"." && name.as_bytes() != b".." {
            names.push(name);
        }
    }

    Ok(names)
}

fn remove_run_folder(store: &OwnedFd, name: &CStr) -> Result<(), StageError> {
    let shown = || name.to_string_lossy().into_owned();

    match rustix::fs::statat(store, name, AtFlags::SYMLINK_NOFOLLOW) {
        Ok(stat) if FileType::from_raw_mode(stat.st_mode) == FileType::Directory => {}
        Ok(_) => return Err(StageError::NoRunFolder(shown())),
        Err(Errno::NOENT) => return Ok(()),
        Err(error) => return Err(error.into()),
    }

    let folder = rustix::fs::openat(store, name, DIR_FLAGS, Mode::empty())?;
    let files = entries(&folder)?;

    for file in &files {
        let stat = rustix::fs::statat(&folder, file, AtFlags::SYMLINK_NOFOLLOW)?;

        if FileType::from_raw_mode(stat.st_mode) != FileType::RegularFile {
            return Err(StageError::NotRegular(file.to_string_lossy().into_owned()));
        }
    }

    for file in &files {
        rustix::fs::unlinkat(&folder, file, AtFlags::empty())?;
    }

    rustix::fs::unlinkat(store, name, AtFlags::REMOVEDIR)?;
    rustix::fs::fsync(store)?;
    Ok(())
}

fn built_folder(root: &Path, uid: u32, home: &Path, run: &BuildRun) -> Result<OwnedFd, StageError> {
    let mut folder = secure::open_dir(root, home)?;
    owned_and_closed(&folder, uid, &home.display().to_string())?;

    for part in AUR_BUILT.split('/').chain([run.as_str()]) {
        folder = rustix::fs::openat(&folder, part, DIR_FLAGS, Mode::empty())?;
        owned_and_closed(&folder, uid, part)?;
    }

    Ok(folder)
}

fn only_file<'files>(
    files: &'files [String],
    package: &PackageName,
) -> Result<&'files String, StageError> {
    let mut found = files
        .iter()
        .filter(|file| package_of(file) == Some(package.as_str()));

    match (found.next(), found.next()) {
        (Some(file), None) => Ok(file),
        (None, _) => Err(StageError::NotBuilt(package.clone())),
        (Some(_), Some(_)) => Err(StageError::Ambiguous(package.clone())),
    }
}

fn copy(built: &OwnedFd, store: &OwnedFd, name: &str, uid: u32) -> Result<(), StageError> {
    let source = rustix::fs::openat(built, name, PACKAGE_READ, Mode::empty())?;
    if FileType::from_raw_mode(rustix::fs::fstat(&source)?.st_mode) != FileType::RegularFile {
        return Err(StageError::NotRegular(name.to_owned()));
    }

    owned_and_closed(&source, uid, name)?;

    let target = rustix::fs::openat(store, name, CREATE_FLAGS, PACKAGE_MODE)?;
    rustix::fs::fchmod(&target, PACKAGE_MODE)?;

    let mut target = File::from(target);
    io::copy(&mut File::from(source), &mut target)?;
    target.sync_all()?;
    Ok(())
}

fn owned_and_closed(fd: &OwnedFd, uid: u32, what: &str) -> Result<(), StageError> {
    let stat = rustix::fs::fstat(fd)?;

    if stat.st_uid != uid {
        return Err(StageError::Foreign(what.to_owned()));
    }

    if stat.st_mode & GROUP_OR_OTHERS_WRITE != 0 {
        return Err(StageError::Shared(what.to_owned()));
    }

    Ok(())
}
