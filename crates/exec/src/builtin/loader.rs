#[cfg(test)]
mod tests;

use std::io;
use std::path::Path;

use toolbox_plan::sdboot::LoaderChange;

use super::FileChange;
use crate::secure;

/// `secure` opens fixed paths only, so each place `esp_place` can name has
/// its own.
const LOADER_CONFS: [(&str, &str); 3] = [
    ("efi", "efi/loader/loader.conf"),
    ("boot", "boot/loader/loader.conf"),
    ("boot/efi", "boot/efi/loader/loader.conf"),
];

const BACKUP: &str = "loader.conf.bak";

/// # Errors
///
/// When no ESP is mounted, a folder on the way is a symlink, the file is
/// a symlink or not UTF-8, or the write fails.
pub fn edit_loader_conf(root: &Path, change: &LoaderChange) -> io::Result<FileChange> {
    let place = crate::sdboot::esp(root)?;
    let path = LOADER_CONFS
        .into_iter()
        .find_map(|(known, path)| (known == place).then_some(path))
        .ok_or_else(|| io::Error::other("the ESP is mounted at no known place"))?;

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
