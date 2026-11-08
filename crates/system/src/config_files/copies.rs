//! The private folder with the copy of a config file the user edits. It
//! may hold what only root can read (`/etc/sudoers`), so folder and file
//! are for the user alone, and the copy goes when the edit ends.

#[cfg(test)]
mod tests;

use std::ffi::OsString;
use std::fs::{self, DirBuilder, OpenOptions, Permissions};
use std::io::{self, Write as _};
use std::os::unix::fs::{DirBuilderExt as _, OpenOptionsExt as _, PermissionsExt as _};
use std::path::{Path, PathBuf};

use toolbox_plan::{ConfigText, EDIT_FOLDER, EditCopy, InvalidConfigText};

/// The last part of [`EDIT_FOLDER`].
const EDIT_NAME: &str = "edit";

const FOLDER_MODE: u32 = 0o700;
const FILE_MODE: u32 = 0o600;

/// What the dry run's stand-in for an editor adds, so the self-test has
/// a change to review without running a program.
const STAND_IN_LINE: &str = "# Edited by the dry run of Garuda Toolbox\n";

/// `<base>/garuda-toolbox/edit`, mode 0700.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditFolder {
    dir: PathBuf,
}

impl EditFolder {
    /// Below `$XDG_RUNTIME_DIR`, which is the user's alone, kept in memory
    /// and emptied at logout. Without one, `edit` in the app's own cache
    /// folder `app_cache` (`~/.cache/garuda-toolbox`).
    #[must_use = "the folder is only named, nothing is created"]
    pub fn from_env<F>(lookup: F, app_cache: &Path) -> Self
    where
        F: Fn(&str) -> Option<OsString>,
    {
        lookup("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute() && path.is_dir())
            .map_or_else(
                || Self {
                    dir: app_cache.join(EDIT_NAME),
                },
                |runtime| Self::new(&runtime),
            )
    }

    #[must_use = "the folder is only named, nothing is created"]
    pub fn new(base: &Path) -> Self {
        Self {
            dir: base.join(EDIT_FOLDER),
        }
    }

    pub fn path(&self) -> &Path {
        &self.dir
    }

    /// Removes every copy. For the start of the app and of an edit: a
    /// copy left by a crash must not stay around.
    ///
    /// # Errors
    ///
    /// When a file cannot be removed. A missing folder is fine.
    pub fn clear(&self) -> io::Result<()> {
        let entries = match fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        };

        for entry in entries {
            fs::remove_file(entry?.path())?;
        }

        Ok(())
    }

    /// Writes `text` as the only copy, named `name` so editors pick their
    /// syntax by it. Folder 0700, file 0600, nothing followed or reused.
    ///
    /// # Errors
    ///
    /// When the folder is a symlink or cannot be made private, the name is
    /// no plain file name, or the write fails.
    pub fn create(&self, name: &str, text: &ConfigText) -> io::Result<EditCopy> {
        DirBuilder::new()
            .recursive(true)
            .mode(FOLDER_MODE)
            .create(&self.dir)?;

        if fs::symlink_metadata(&self.dir)?.file_type().is_symlink() {
            return Err(io::Error::other("the edit folder is a link"));
        }

        fs::set_permissions(&self.dir, Permissions::from_mode(FOLDER_MODE))?;
        self.clear()?;

        let copy = self
            .dir
            .join(name)
            .to_str()
            .ok_or_else(|| io::Error::other("the edit folder is no text"))
            .and_then(|path| EditCopy::new(path).map_err(io::Error::other))?;

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(FILE_MODE)
            .open(copy.as_str())?;

        file.write_all(text.as_str().as_bytes())?;
        file.sync_all()?;
        Ok(copy)
    }

    /// What the copy holds now.
    ///
    /// # Errors
    ///
    /// When the copy cannot be read, or the editor left something that is
    /// no text or too large to write back.
    pub fn read(&self, copy: &EditCopy) -> io::Result<Result<ConfigText, InvalidConfigText>> {
        self.owned(copy)?;
        Ok(ConfigText::from_bytes(fs::read(copy.as_str())?))
    }

    /// Removes the copy. A missing one is fine.
    ///
    /// # Errors
    ///
    /// When the copy cannot be removed.
    pub fn remove(&self, copy: &EditCopy) -> io::Result<()> {
        self.owned(copy)?;

        match fs::remove_file(copy.as_str()) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }

    /// Plays the editor in a dry run: appends one comment line.
    ///
    /// # Errors
    ///
    /// When the copy cannot be written.
    pub fn stand_in_edit(&self, copy: &EditCopy) -> io::Result<()> {
        self.owned(copy)?;

        OpenOptions::new()
            .append(true)
            .open(copy.as_str())?
            .write_all(STAND_IN_LINE.as_bytes())
    }

    /// A copy of another folder is none of this one's business.
    fn owned(&self, copy: &EditCopy) -> io::Result<()> {
        if Path::new(copy.as_str()).parent() == Some(self.dir.as_path()) {
            Ok(())
        } else {
            Err(io::Error::other("the copy is not in the edit folder"))
        }
    }
}
