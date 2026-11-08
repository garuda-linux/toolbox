//! File writes as root that cannot be redirected by a symlink. A folder is
//! opened with `openat2`, which refuses every symlink on the way and every
//! step out of the root, and files are created, linked and renamed relative
//! to that folder's descriptor (`toolbox-fs`), so swapping a folder for a
//! symlink between check and write leads nowhere.

#[cfg(test)]
mod tests;

use std::fs::File;
use std::io::{self, Read as _};
use std::os::fd::OwnedFd;
use std::path::{Component, Path};

use rustix::fs::{AtFlags, FileType, Mode, OFlags, ResolveFlags};
use rustix::io::Errno;
use toolbox_fs::{Owner, Permissions, Replace};

const DIR_FLAGS: OFlags = OFlags::DIRECTORY
    .union(OFlags::RDONLY)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::CLOEXEC);

const RESOLVE: ResolveFlags = ResolveFlags::NO_SYMLINKS.union(ResolveFlags::BENEATH);

const READ_FLAGS: OFlags = OFlags::RDONLY
    .union(OFlags::NOFOLLOW)
    .union(OFlags::CLOEXEC);

/// 0755, like the folders packages ship below `/etc`.
const DIR_MODE: Mode = Mode::RWXU
    .union(Mode::RGRP)
    .union(Mode::XGRP)
    .union(Mode::ROTH)
    .union(Mode::XOTH);

const PERMISSION_BITS: u32 = 0o7777;

/// A system file the helper writes: mode 0644 whatever the umask, so
/// pacman run as any user can read it.
const SYSTEM_FILE: Replace<'static> = Replace::new(Permissions::Exactly(0o644));

/// FAT refuses `chmod` under a `fmask`, so its mount decides the mode.
const ESP_FILE: Replace<'static> = Replace::new(Permissions::Umask(0o644));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attributes {
    pub mode: u32,
    pub owner: u32,
    pub group: u32,
}

#[derive(Debug)]
pub struct Regular {
    pub content: Vec<u8>,
    pub attributes: Attributes,
}

#[derive(Debug)]
pub enum Unreadable {
    Missing,
    /// A symlink, which is never followed.
    Link,
    NotRegular,
    TooLarge,
    Failed(io::Error),
}

impl Attributes {
    #[must_use = "a replace is only described, nothing is written"]
    pub const fn replace(self) -> Replace<'static> {
        Replace::new(Permissions::Exactly(self.mode)).owned_by(Owner {
            user: self.owner,
            group: self.group,
        })
    }
}

impl From<Errno> for Unreadable {
    fn from(error: Errno) -> Self {
        match error {
            Errno::NOENT => Self::Missing,
            // What an open without following answers for a symlink.
            Errno::LOOP | Errno::NOTDIR => Self::Link,
            // What an open answers for a socket.
            Errno::NXIO => Self::NotRegular,
            other => Self::Failed(other.into()),
        }
    }
}

/// Opens `dir` (relative, plain components only) below `root` without
/// following a symlink on the way. `root` itself is trusted: `/` in the
/// helper, a temporary directory in tests.
///
/// # Errors
///
/// When `dir` is not a plain relative path, a component is missing, is a
/// symlink or is not a directory.
pub fn open_dir(root: &Path, dir: &Path) -> io::Result<OwnedFd> {
    plain(dir)?;

    let fd = rustix::fs::open(root, DIR_FLAGS, Mode::empty())?;

    if dir.as_os_str().is_empty() {
        return Ok(fd);
    }

    Ok(rustix::fs::openat2(
        &fd,
        dir,
        DIR_FLAGS,
        Mode::empty(),
        RESOLVE,
    )?)
}

/// Like [`open_dir`], but creates missing components (mode 0755) on the
/// way. An existing symlink is still refused.
///
/// # Errors
///
/// When `dir` is not a plain relative path, a component cannot be created
/// or is a symlink or not a directory.
pub fn create_dirs(root: &Path, dir: &Path) -> io::Result<OwnedFd> {
    plain(dir)?;

    let mut fd = rustix::fs::open(root, DIR_FLAGS, Mode::empty())?;

    // One step at a time: a folder is made below the descriptor of its
    // parent, never through a path a symlink could bend.
    for name in dir.components().map(Component::as_os_str) {
        match rustix::fs::mkdirat(&fd, name, DIR_MODE) {
            Ok(()) | Err(Errno::EXIST) => {}
            Err(error) => return Err(error.into()),
        }

        fd = rustix::fs::openat2(&fd, name, DIR_FLAGS, Mode::empty(), RESOLVE)?;
    }

    Ok(fd)
}

/// `openat2` would accept `a/../b` below the root. The callers name
/// fixed folders, so anything but plain names is a mistake.
fn plain(dir: &Path) -> io::Result<()> {
    if dir
        .components()
        .all(|part| matches!(part, Component::Normal(_)))
    {
        return Ok(());
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("{}: not a plain relative path", dir.display()),
    ))
}

/// The content of `name` in `dir`, `None` when it does not exist. A
/// symlink is refused rather than followed.
///
/// # Errors
///
/// When `name` is a symlink or cannot be read.
pub fn read_file(dir: &OwnedFd, name: &str) -> io::Result<Option<Vec<u8>>> {
    let fd = match rustix::fs::openat(dir, name, READ_FLAGS, Mode::empty()) {
        Ok(fd) => fd,
        Err(Errno::NOENT) => return Ok(None),
        Err(error) => return Err(error.into()),
    };

    let mut content = Vec::new();
    File::from(fd).read_to_end(&mut content)?;
    Ok(Some(content))
}

/// [`read_file`] as UTF-8 text.
///
/// # Errors
///
/// As [`read_file`], and when the content is not UTF-8.
pub fn read_text(dir: &OwnedFd, name: &str) -> io::Result<Option<String>> {
    read_file(dir, name)?
        .map(|bytes| {
            String::from_utf8(bytes)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
        })
        .transpose()
}

/// # Errors
///
/// When `path` has no folder or no file name.
pub fn split(path: &'static str) -> io::Result<(&'static Path, &'static str)> {
    let target = Path::new(path);
    let (Some(folder), Some(name)) = (
        target.parent(),
        target.file_name().and_then(|name| name.to_str()),
    ) else {
        return Err(io::Error::other(format!("{path} has no folder")));
    };

    Ok((folder, name))
}

/// # Errors
///
/// As [`split`] and [`open_dir`].
pub fn open_parent(root: &Path, path: &'static str) -> io::Result<(OwnedFd, &'static str)> {
    let (folder, name) = split(path)?;

    Ok((open_dir(root, folder)?, name))
}

/// Removes `name` from `dir`. A symlink is removed itself, never its
/// target. Returns whether there was something to remove.
///
/// # Errors
///
/// When `name` exists but cannot be removed.
pub fn remove_file(dir: &OwnedFd, name: &str) -> io::Result<bool> {
    match rustix::fs::unlinkat(dir, name, AtFlags::empty()) {
        Ok(()) => {}
        Err(Errno::NOENT) => return Ok(false),
        Err(error) => return Err(error.into()),
    }

    rustix::fs::fsync(dir)?;
    Ok(true)
}

/// Replaces `name` in `dir` with `content` as a [`SYSTEM_FILE`], keeping the
/// old file as `backup` when one is named.
///
/// # Errors
///
/// When the write or the sync of the folder fails.
pub fn replace_system_file(
    dir: &OwnedFd,
    name: &str,
    content: &[u8],
    backup: Option<&str>,
) -> io::Result<bool> {
    let how = backup.map_or(SYSTEM_FILE, |backup| SYSTEM_FILE.with_backup(backup));

    toolbox_fs::replace_at(dir, name, content, &how)?.synced()
}

/// FAT has no hard links, so `old` becomes the backup as a copy.
///
/// # Errors
///
/// When a write or the sync of the folder fails.
pub fn replace_esp_file(
    dir: &OwnedFd,
    name: &str,
    content: &[u8],
    backup: Option<(&str, &[u8])>,
) -> io::Result<()> {
    if let Some((backup, old)) = backup {
        toolbox_fs::replace_at(dir, backup, old, &ESP_FILE)?.synced()?;
    }

    toolbox_fs::replace_at(dir, name, content, &ESP_FILE)?
        .synced()
        .map(drop)
}

/// # Errors
///
/// [`Unreadable`] for anything but a regular file of at most `limit`
/// bytes that can be read.
pub fn read_regular(dir: &OwnedFd, name: &str, limit: usize) -> Result<Regular, Unreadable> {
    // Looked at before it is opened: opening a device or a FIFO can block
    // or do things to it. The look after the open is the one that counts.
    let seen =
        rustix::fs::statat(dir, name, AtFlags::SYMLINK_NOFOLLOW).map_err(Unreadable::from)?;

    refuse_other_kinds(FileType::from_raw_mode(seen.st_mode))?;

    let fd = rustix::fs::openat(dir, name, READ_FLAGS.union(OFlags::NONBLOCK), Mode::empty())
        .map_err(Unreadable::from)?;

    let stat = rustix::fs::fstat(&fd).map_err(Unreadable::from)?;

    refuse_other_kinds(FileType::from_raw_mode(stat.st_mode))?;

    let cap = u64::try_from(limit).unwrap_or(u64::MAX);
    let mut content = Vec::new();
    let read = File::from(fd)
        .take(cap.saturating_add(1))
        .read_to_end(&mut content)
        .map_err(Unreadable::Failed)?;

    if read > limit {
        return Err(Unreadable::TooLarge);
    }

    Ok(Regular {
        content,
        attributes: Attributes {
            mode: stat.st_mode & PERMISSION_BITS,
            owner: stat.st_uid,
            group: stat.st_gid,
        },
    })
}

/// A symlink is not followed, so it is none.
pub fn is_regular(dir: &OwnedFd, name: &str) -> bool {
    rustix::fs::statat(dir, name, AtFlags::SYMLINK_NOFOLLOW)
        .is_ok_and(|stat| FileType::from_raw_mode(stat.st_mode) == FileType::RegularFile)
}

/// The names of the regular files directly in `dir`, sorted. Symlinks,
/// folders and names that are no UTF-8 are left out.
///
/// # Errors
///
/// When the folder cannot be listed.
pub fn regular_files(dir: &OwnedFd) -> io::Result<Vec<String>> {
    let mut names = Vec::new();

    for entry in rustix::fs::Dir::read_from(dir)? {
        let entry = entry?;
        let Ok(name) = entry.file_name().to_str() else {
            continue;
        };

        if is_regular(dir, name) {
            names.push(name.to_owned());
        }
    }

    names.sort();
    Ok(names)
}

const fn refuse_other_kinds(kind: FileType) -> Result<(), Unreadable> {
    match kind {
        FileType::RegularFile => Ok(()),
        FileType::Symlink => Err(Unreadable::Link),
        FileType::Directory
        | FileType::Fifo
        | FileType::Socket
        | FileType::CharacterDevice
        | FileType::BlockDevice
        | FileType::Unknown => Err(Unreadable::NotRegular),
    }
}
