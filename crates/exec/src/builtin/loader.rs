#[cfg(test)]
mod tests;

use std::io;
use std::path::Path;

use toolbox_plan::sdboot::LoaderChange;

use super::FileChange;
use crate::secure;

const BACKUP: &str = "loader.conf.bak";

/// # Errors
///
/// When no ESP is mounted, a folder on the way is a symlink, the file is
/// a symlink or not UTF-8, or the write fails.
pub fn edit_loader_conf(root: &Path, change: &LoaderChange) -> io::Result<FileChange> {
    let path = crate::sdboot::esp(root)?.loader_conf();

    let (dir, name) = secure::open_parent(root, path)?;
    let current = secure::read_text(&dir, name)?;
    let before = current.as_deref().unwrap_or_default();
    let wanted = change.apply(before);

    if wanted == before {
        return Ok(FileChange::Unchanged);
    }

    // systemd-boot runs on its defaults without the file, so there is no
    // old one to keep then.
    let backup = current.as_deref().map(|old| (BACKUP, old.as_bytes()));

    secure::replace_esp_file(&dir, name, wanted.as_bytes(), backup)?;
    Ok(FileChange::Written)
}
