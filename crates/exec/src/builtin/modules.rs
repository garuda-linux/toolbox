//! The one modprobe file the app owns. Files of packages
//! (`/usr/lib/modprobe.d`) and of the administrator are never written: a
//! package update or the administrator owns them.

#[cfg(test)]
mod tests;

use std::io;
use std::path::Path;

use toolbox_plan::{
    KernelModule, MODPROBE_BLACKLIST, Toggle, blacklisted_modules, set_blacklisted,
};

use super::FileChange;
use crate::secure;

/// Adds `module` to the app's blacklist file or takes it out. Every other
/// line of the file stays. The file goes when its last entry does.
///
/// # Errors
///
/// When a folder on the way is a symlink, the file is not UTF-8, or the
/// write fails.
pub fn edit_modprobe_blacklist(
    root: &Path,
    module: &KernelModule,
    wish: Toggle,
) -> io::Result<FileChange> {
    let on = wish.is_on();
    let (folder, name) = secure::split(MODPROBE_BLACKLIST)?;

    let dir = if on {
        secure::create_dirs(root, folder)?
    } else {
        match secure::open_dir(root, folder) {
            Ok(dir) => dir,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(FileChange::Unchanged);
            }
            Err(error) => return Err(error),
        }
    };

    let current = secure::read_text(&dir, name)?.unwrap_or_default();

    let wanted = set_blacklisted(&current, module, wish);

    if wanted == current {
        return Ok(FileChange::Unchanged);
    }

    if blacklisted_modules(&wanted).is_empty() {
        secure::remove_file(&dir, name)?;
        return Ok(FileChange::Removed);
    }

    secure::replace_system_file(&dir, name, wanted.as_bytes(), None)?;
    Ok(FileChange::Written)
}
