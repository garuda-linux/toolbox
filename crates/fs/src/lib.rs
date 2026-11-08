//! Replacing a file in one step: a temporary file beside the target,
//! written and synced, then renamed over it. A crash leaves the old file
//! or the whole new one, never half of one.
//!
//! [`replace_at`] works below an open folder, so a folder swapped for a
//! symlink after it was opened cannot redirect the write. The root helper
//! uses it with folders opened without following links. [`replace`] is
//! the form for the user's own files, which follows links on the way.

#[cfg(test)]
mod tests;

use std::fs::File;
use std::io::{self, Write as _};
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use rustix::fs::{AtFlags, Mode, OFlags};
use rustix::io::Errno;

const NEW_FILE_FLAGS: OFlags = OFlags::WRONLY
    .union(OFlags::CREATE)
    .union(OFlags::EXCL)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::CLOEXEC);

const DIR_FLAGS: OFlags = OFlags::DIRECTORY
    .union(OFlags::RDONLY)
    .union(OFlags::CLOEXEC);

const PATH_FLAGS: OFlags = OFlags::DIRECTORY.union(OFlags::PATH).union(OFlags::CLOEXEC);

/// The content is never readable by more users than the result.
const PRIVATE_MODE: Mode = Mode::RUSR.union(Mode::WUSR);

/// A kept mode never carries set-id or sticky bits over to a new file.
const KEPT_BITS: u32 = 0o777;

/// Keeps two temporary names apart that read the same clock value.
static SERIAL: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Permissions {
    /// Exactly these bits, whatever the umask.
    Exactly(u32),
    /// What the umask leaves of these bits, like a file an editor saves.
    Umask(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Owner {
    pub user: u32,
    pub group: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Replace<'name> {
    permissions: Permissions,
    keep_mode: bool,
    owner: Option<Owner>,
    backup: Option<&'name str>,
}

#[derive(Debug)]
pub struct Replaced {
    pub backup_kept: bool,
    /// The sync of the folder after the rename. Without it a crash may
    /// still bring back the old file.
    pub folder_sync: io::Result<()>,
}

impl<'name> Replace<'name> {
    #[must_use = "a replace is only described, nothing is written"]
    pub const fn new(permissions: Permissions) -> Self {
        Self {
            permissions,
            keep_mode: false,
            owner: None,
            backup: None,
        }
    }

    /// An existing target keeps its mode, the given one is for a new file.
    #[must_use = "a replace is only described, nothing is written"]
    pub const fn keeping_mode(mut self) -> Self {
        self.keep_mode = true;
        self
    }

    #[must_use = "a replace is only described, nothing is written"]
    pub const fn owned_by(mut self, owner: Owner) -> Self {
        self.owner = Some(owner);
        self
    }

    /// The old file is hard linked under `backup` first, so the target
    /// never goes missing. A symlinked target is kept as that symlink.
    #[must_use = "a replace is only described, nothing is written"]
    pub const fn with_backup(mut self, backup: &'name str) -> Self {
        self.backup = Some(backup);
        self
    }
}

impl Replaced {
    /// # Errors
    ///
    /// When the folder could not be synced. The new file is in place.
    pub fn synced(self) -> io::Result<bool> {
        self.folder_sync.map(|()| self.backup_kept)
    }
}

/// A symlink at `name` is replaced itself, never followed.
///
/// # Errors
///
/// When the mode cannot be read, or writing, owning, linking or renaming
/// fails. The temporary file is removed again.
pub fn replace_at<Fd>(
    dir: Fd,
    name: &str,
    content: &[u8],
    how: &Replace<'_>,
) -> io::Result<Replaced>
where
    Fd: AsFd,
{
    let dir = dir.as_fd();
    let permissions = permissions(dir, name, how)?;
    let created = match permissions {
        Permissions::Exactly(_) => PRIVATE_MODE,
        Permissions::Umask(mode) => Mode::from_raw_mode(mode),
    };

    let temp = temp_name(name);
    let fd = rustix::fs::openat(dir, temp.as_str(), NEW_FILE_FLAGS, created)?;

    let result = fill(fd, content, permissions, how.owner)
        .and_then(|()| {
            how.backup
                .map_or(Ok(false), |backup| keep_old(dir, name, backup))
        })
        .and_then(|kept| {
            rustix::fs::renameat(dir, temp.as_str(), dir, name)?;
            Ok(kept)
        });

    let backup_kept = match result {
        Ok(kept) => kept,
        Err(error) => {
            // Best effort: the error worth reporting is the write's.
            rustix::fs::unlinkat(dir, temp.as_str(), AtFlags::empty()).ok();
            return Err(error);
        }
    };

    Ok(Replaced {
        backup_kept,
        folder_sync: rustix::fs::fsync(dir).map_err(io::Error::from),
    })
}

/// Links on the way to the folder are followed.
///
/// # Errors
///
/// When `path` has no file name that is text, a folder cannot be made or
/// opened, or the replace fails.
pub fn replace(path: &Path, content: &[u8], how: &Replace<'_>) -> io::Result<Replaced> {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{}: no file name", path.display()),
        ));
    };

    let folder = path
        .parent()
        .filter(|folder| !folder.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    std::fs::create_dir_all(folder)?;

    // A folder the user may write but not read still takes the rename.
    // Only its sync then fails, which the caller hears.
    let dir: OwnedFd = match rustix::fs::open(folder, DIR_FLAGS, Mode::empty()) {
        Err(Errno::ACCESS) => rustix::fs::open(folder, PATH_FLAGS, Mode::empty())?,
        opened => opened?,
    };

    replace_at(&dir, name, content, how)
}

fn permissions(dir: BorrowedFd<'_>, name: &str, how: &Replace<'_>) -> io::Result<Permissions> {
    if !how.keep_mode {
        return Ok(how.permissions);
    }

    match rustix::fs::statat(dir, name, AtFlags::empty()) {
        Ok(stat) => Ok(Permissions::Exactly(stat.st_mode & KEPT_BITS)),
        Err(Errno::NOENT) => Ok(how.permissions),
        Err(error) => Err(error.into()),
    }
}

fn fill(
    fd: OwnedFd,
    content: &[u8],
    permissions: Permissions,
    owner: Option<Owner>,
) -> io::Result<()> {
    // Before the mode, since a new owner clears set-id bits.
    if let Some(owner) = owner {
        std::os::unix::fs::fchown(&fd, Some(owner.user), Some(owner.group))?;
    }

    // The umask (pkexec keeps the caller's) narrowed the creation mode.
    if let Permissions::Exactly(mode) = permissions {
        rustix::fs::fchmod(&fd, Mode::from_raw_mode(mode))?;
    }

    let mut file = File::from(fd);

    file.write_all(content)?;
    file.sync_all()
}

/// Hard links the current `name` to `backup` through a temporary name, so
/// an existing backup is replaced in one rename. A link never follows a
/// symlink.
fn keep_old(dir: BorrowedFd<'_>, name: &str, backup: &str) -> io::Result<bool> {
    let temp = temp_name(backup);

    match rustix::fs::linkat(dir, name, dir, temp.as_str(), AtFlags::empty()) {
        Ok(()) => {}
        Err(Errno::NOENT) => return Ok(false),
        Err(error) => return Err(error.into()),
    }

    if let Err(error) = rustix::fs::renameat(dir, temp.as_str(), dir, backup) {
        rustix::fs::unlinkat(dir, temp.as_str(), AtFlags::empty()).ok();
        return Err(error.into());
    }

    Ok(true)
}

fn temp_name(name: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |age| age.as_nanos());

    format!(
        ".{name}.tmp-{}-{nanos}-{}",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    )
}
