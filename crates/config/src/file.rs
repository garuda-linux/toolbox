//! Writing the app's own files so a crash never leaves half of one.

#[cfg(test)]
mod tests;

use std::io;
use std::path::{Path, PathBuf};

use toolbox_fs::{Permissions, Replace};

/// Links followed before a path counts as a loop, as `SYMLOOP_MAX` on Linux.
const MAX_LINKS: usize = 40;

const NEW_FILE: Replace<'static> = Replace::new(Permissions::Exactly(0o644));

/// Replaces the file at `path` with `bytes` and creates missing folders.
///
/// A symlink at `path` stays: its target gets the new file, so settings
/// kept in a dotfiles repository remain linked. The new file keeps the
/// destination's mode, or 0644 when there is none yet, and the folder is
/// synced after the rename, so a crash leaves the old file or the whole
/// new one.
///
/// # Errors
///
/// When a link cannot be read or loops, or the folder, the temporary file
/// or the rename fails. A failed folder sync only logs: the file is in
/// place by then.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let path = resolve_links(path)?;
    let replaced = toolbox_fs::replace(&path, bytes, &NEW_FILE.keeping_mode())?;

    if let Err(error) = replaced.folder_sync {
        tracing::warn!(
            "saved {} but cannot sync its folder: {error}",
            path.display()
        );
    }

    Ok(())
}

/// The file a chain of symlinks at `path` ends in, even when it does not
/// exist yet. Relative targets count from the link's own folder.
fn resolve_links(path: &Path) -> io::Result<PathBuf> {
    let mut current = path.to_owned();

    for _ in 0..MAX_LINKS {
        match std::fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => {
                let target = std::fs::read_link(&current)?;
                let base = current.parent().unwrap_or_else(|| Path::new("."));
                current = base.join(target);
            }
            Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
            _ => return Ok(current),
        }
    }

    Err(io::Error::other(format!(
        "too many links at {}",
        path.display()
    )))
}
