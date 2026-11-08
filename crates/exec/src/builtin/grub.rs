//! `/etc/default/grub`, the one file behind the Boot options page. The
//! edit is computed in `toolbox-plan` and written through the `secure`
//! module: only the wanted variables change, the write is atomic, and a
//! symlink planted on the way leads nowhere.

#[cfg(test)]
mod tests;

use std::io;
use std::path::Path;

use toolbox_plan::{GRUB_DEFAULTS, GRUB_DEFAULTS_BACKUP, GrubSettings};

use super::FileChange;
use crate::secure;

/// Sets `settings` in `/etc/default/grub` below `root` and keeps the
/// file as it was as `grub.bak` beside it.
///
/// # Errors
///
/// When the file is missing (GRUB is not the boot loader, and the helper
/// never creates the file), a folder on the way is a symlink, the file is
/// a symlink or not UTF-8, or the write fails.
pub fn edit_grub_defaults(root: &Path, settings: &GrubSettings) -> io::Result<FileChange> {
    let (dir, name) = secure::open_parent(root, GRUB_DEFAULTS)?;
    let Some(current) = secure::read_text(&dir, name)? else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "the file does not exist, so GRUB is not the boot loader of this system",
        ));
    };

    let wanted = settings.apply(&current);

    super::rewritten(&dir, name, &current, &wanted, Some(GRUB_DEFAULTS_BACKUP))
}
