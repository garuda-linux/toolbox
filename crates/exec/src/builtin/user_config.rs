//! Edits of config files in the user's home. They run as the user, so the
//! kernel's permissions already keep them to what the user may change.
//! Symlinks are followed like any program of the user would: dotfile
//! managers link `~/.config` files, and replacing such a link with a plain
//! file would cut it off. The write itself is atomic and keeps the mode.

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::{fs, io};

use toolbox_fs::{Permissions, Replace};
use toolbox_plan::{Rewrite, UserConfigEdit};

use super::settings::FileChange;

/// New files get what the user's umask leaves of this, like a file saved
/// by an editor.
const NEW_FILE: Replace<'static> = Replace::new(Permissions::Umask(0o666));

/// # Errors
///
/// When the file cannot be read or written, or the edit refuses its
/// content (not text, not a JSON object).
pub fn edit_user_config(home: &Path, edit: &UserConfigEdit, on: bool) -> io::Result<FileChange> {
    let target = resolve(&home.join(edit.path().as_str()))?;
    let current = read_optional(&target)?;
    let rewrite = edit
        .rewrite(current.as_deref(), on)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    match rewrite {
        Rewrite::Unchanged => Ok(FileChange::Unchanged),
        Rewrite::Remove => {
            fs::remove_file(&target)?;
            Ok(FileChange::Removed)
        }
        Rewrite::Write(content) => {
            toolbox_fs::replace(&target, content.as_bytes(), &NEW_FILE.keeping_mode())?.synced()?;
            Ok(FileChange::Written)
        }
    }
}

/// The file a symlink points to, so the link stays. A dangling link is
/// refused: where it should lead is not for the toolbox to guess.
pub(super) fn resolve(path: &Path) -> io::Result<PathBuf> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => fs::canonicalize(path),
        Ok(_) => Ok(path.to_path_buf()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(path.to_path_buf()),
        Err(error) => Err(error),
    }
}

fn read_optional(path: &Path) -> io::Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}
