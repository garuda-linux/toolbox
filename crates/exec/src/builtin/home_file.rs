//! The write-back of an edited file in the user's home. It runs as the
//! user, so the kernel's permissions keep it to what the user may change.
//! Like a save of a root-owned file it keeps the old version and writes
//! only while the file is still the version the edit started from.

#[cfg(test)]
mod tests;

use std::fs::{self, DirBuilder};
use std::io;
use std::os::unix::fs::DirBuilderExt as _;
use std::path::Path;

use toolbox_fs::{Permissions, Replace};
use toolbox_plan::{Fingerprint, HomeWrite};

use super::user_config::resolve;

/// Backups stay readable by the user alone: config files may hold keys.
const BACKUP: Replace<'static> = Replace::new(Permissions::Exactly(0o600));
const BACKUP_FOLDER_MODE: u32 = 0o700;
/// Below the home, as `$XDG_STATE_HOME` has it by default.
const STATE: &str = ".local/state";

/// Replaces the file `write` names below `home` and keeps its old text in
/// the backup folder.
///
/// # Errors
///
/// When the file is missing, changed since it was read, or cannot be
/// read or written.
pub fn write_home_file(home: &Path, write: &HomeWrite) -> io::Result<()> {
    let target = resolve(&home.join(write.path.as_str()))?;
    let current = fs::read(&target)?;

    if Fingerprint::of(&current) != write.original {
        return Err(io::Error::other(
            "the file changed since it was read, so nothing was written",
        ));
    }

    let backup = home.join(STATE).join(write.backup());
    if let Some(folder) = backup.parent() {
        DirBuilder::new()
            .recursive(true)
            .mode(BACKUP_FOLDER_MODE)
            .create(folder)?;
    }

    toolbox_fs::replace(&backup, &current, &BACKUP)?.synced()?;

    let keep = Replace::new(Permissions::Umask(0o666)).keeping_mode();
    toolbox_fs::replace(&target, write.content.as_str().as_bytes(), &keep)?.synced()?;
    Ok(())
}
