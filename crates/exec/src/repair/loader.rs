//! Which boot loader the mounted system starts with, seen from its own
//! files, and the steps of the repair for it.

use std::io;
use std::path::Path;
use std::string::FromUtf8Error;

use toolbox_plan::procedure::{Invocation, Program, uki_rebuilds};
use toolbox_plan::{
    InstalledLoader, InstalledSystem, LoaderRefusal, MAX_CONFIG_BYTES, REPAIR_ROOT, RepairWork,
};

use super::absent_as_none;
use crate::secure::{self, Unreadable};

/// Why the steps of the boot loader cannot be made.
#[derive(Debug, thiserror::Error)]
pub enum LoaderStepsError {
    #[error("cannot tell the boot loader of the mounted system: {0}")]
    Unknown(#[source] io::Error),
    #[error(transparent)]
    Refused(#[from] LoaderRefusal),
}

/// A file of the mounted system the boot loader is read from, refused.
#[derive(Debug, thiserror::Error)]
enum RefusedFile {
    #[error("{name} is not UTF-8 text: {error}")]
    NotUtf8 { name: String, error: FromUtf8Error },
    #[error("{name} is a symlink")]
    Link { name: String },
    #[error("{name} is not a regular file")]
    NotRegular { name: String },
    #[error("{name} is larger than {max} bytes")]
    TooLarge { name: String, max: usize },
}

/// # Errors
///
/// When `/etc/kernel/install.conf` or `/etc/fstab` of the mounted system
/// is there but cannot be read.
pub(super) fn installed_loader(root: &Path) -> io::Result<InstalledLoader> {
    let install_conf = text(root, "etc/kernel", "install.conf")?;
    let fstab = text(root, "etc", "fstab")?;

    Ok(InstalledLoader::of(
        install_conf.as_deref(),
        fstab.as_deref(),
    ))
}

/// # Errors
///
/// [`LoaderStepsError::Unknown`] when the boot loader cannot be told,
/// [`LoaderStepsError::Refused`] when the work does not fit it.
pub fn boot_loader_steps(
    root: &Path,
    system: &InstalledSystem,
    work: &RepairWork,
) -> Result<Vec<Invocation>, LoaderStepsError> {
    let loader = installed_loader(root).map_err(LoaderStepsError::Unknown)?;

    Ok(toolbox_plan::boot_loader_steps(system, work, loader)?)
}

/// Whether the mounted system has `program` at its path, not through a
/// symlink on the way.
///
/// # Errors
///
/// When the program's folder is there but cannot be read.
pub fn has(root: &Path, program: Program) -> io::Result<bool> {
    let path = Path::new(program.path());
    let folder = path
        .parent()
        .and_then(|folder| folder.strip_prefix("/").ok())
        .unwrap_or(path);

    let dir = absent_as_none(secure::open_dir(root, &Path::new(REPAIR_ROOT).join(folder)))?;

    Ok(dir.is_some_and(|dir| secure::is_regular(&dir, program.name())))
}

/// Garuda's own script or kernel-install itself.
///
/// # Errors
///
/// As [`has`].
pub(super) fn has_uki_builder(root: &Path) -> io::Result<bool> {
    for command in uki_rebuilds() {
        if has(root, command.program())? {
            return Ok(true);
        }
    }

    Ok(false)
}

/// The chosen disk is not trusted: a FIFO there would block the helper and
/// a device would be read until memory runs out, before any cleanup.
fn text(root: &Path, folder: &str, name: &str) -> io::Result<Option<String>> {
    let Some(dir) = absent_as_none(secure::open_dir(root, &Path::new(REPAIR_ROOT).join(folder)))?
    else {
        return Ok(None);
    };

    let refused = |reason: RefusedFile| io::Error::new(io::ErrorKind::InvalidData, reason);
    let name_of = || name.to_owned();

    match secure::read_regular(&dir, name, MAX_CONFIG_BYTES) {
        Ok(file) => String::from_utf8(file.content).map(Some).map_err(|error| {
            refused(RefusedFile::NotUtf8 {
                name: name_of(),
                error,
            })
        }),
        Err(Unreadable::Missing) => Ok(None),
        Err(Unreadable::Link) => Err(refused(RefusedFile::Link { name: name_of() })),
        Err(Unreadable::NotRegular) => Err(refused(RefusedFile::NotRegular { name: name_of() })),
        Err(Unreadable::TooLarge) => Err(refused(RefusedFile::TooLarge {
            name: name_of(),
            max: MAX_CONFIG_BYTES,
        })),
        Err(Unreadable::Failed(error)) => Err(error),
    }
}
