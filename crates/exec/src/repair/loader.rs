//! Which boot loader the mounted system starts with, seen from its own
//! files, and the steps of the repair for it.

use std::io;
use std::path::Path;

use toolbox_plan::procedure::{Invocation, Program, uki_rebuilds};
use toolbox_plan::{InstalledLoader, InstalledSystem, REPAIR_ROOT, RepairWork};

use super::absent_as_none;
use crate::secure;

/// # Errors
///
/// When `/etc/kernel/install.conf` or `/etc/fstab` of the mounted system
/// is there but cannot be read.
pub fn installed_loader(root: &Path) -> io::Result<InstalledLoader> {
    let install_conf = text(root, "etc/kernel", "install.conf")?;
    let fstab = text(root, "etc", "fstab")?;

    Ok(InstalledLoader::of(
        install_conf.as_deref(),
        fstab.as_deref(),
    ))
}

/// # Errors
///
/// The reason, in words for the Console, when the boot loader cannot be
/// told or the work does not fit it.
pub fn boot_loader_steps(
    root: &Path,
    system: &InstalledSystem,
    work: &RepairWork,
) -> Result<Vec<Invocation>, String> {
    let loader = installed_loader(root)
        .map_err(|error| format!("cannot tell the boot loader of the mounted system: {error}"))?;

    toolbox_plan::boot_loader_steps(system, work, loader).map_err(str::to_owned)
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
pub fn has_uki_builder(root: &Path) -> io::Result<bool> {
    for command in uki_rebuilds() {
        if has(root, command.program())? {
            return Ok(true);
        }
    }

    Ok(false)
}

fn text(root: &Path, folder: &str, name: &str) -> io::Result<Option<String>> {
    absent_as_none(secure::open_dir(root, &Path::new(REPAIR_ROOT).join(folder)))?
        .map_or(Ok(None), |dir| secure::read_text(&dir, name))
}
