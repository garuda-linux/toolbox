//! Where the log file goes: the state folder, or without a home a private
//! folder below the system temporary folder.

#[cfg(test)]
mod tests;

use std::os::unix::fs::{DirBuilderExt as _, MetadataExt as _};
use std::path::{Path, PathBuf};

use toolbox_config::Paths;

#[derive(Debug, thiserror::Error)]
enum NotPrivate {
    #[error("{0}")]
    Unreadable(std::io::Error),
    #[error("not a folder")]
    NoFolder,
    #[error("owned by uid {0}")]
    Owner(u32),
    #[error("mode {0:o}")]
    Writable(u32),
}

/// The folder for the log file. `None` when the temporary fallback is not
/// safe to use, the log then goes to stderr alone.
pub fn log_dir(paths: &Paths) -> Option<PathBuf> {
    paths
        .own_state_dir()
        .or_else(|| temporary(&std::env::temp_dir().join("garuda-toolbox")))
}

fn temporary(dir: &Path) -> Option<PathBuf> {
    match private_folder(dir) {
        Ok(()) => Some(dir.to_owned()),
        Err(reason) => {
            tracing::warn!(
                "{} is not a private folder ({reason}), logging to stderr only",
                dir.display()
            );
            None
        }
    }
}

/// Creates `dir` as 0700 when it is missing, then checks what is there.
/// The temporary folder is shared by every user: another one may have made
/// `dir` first, as a link to their own place or open for writing, and
/// would then read or swap the log. So only a real folder of ours that
/// nobody else can write counts. The check never follows a link.
fn private_folder(dir: &Path) -> Result<(), NotPrivate> {
    if let Err(error) = std::fs::DirBuilder::new().mode(0o700).create(dir)
        && error.kind() != std::io::ErrorKind::AlreadyExists
    {
        return Err(NotPrivate::Unreadable(error));
    }

    let meta = std::fs::symlink_metadata(dir).map_err(NotPrivate::Unreadable)?;

    if !meta.file_type().is_dir() {
        return Err(NotPrivate::NoFolder);
    }

    if meta.uid() != rustix::process::getuid().as_raw() {
        return Err(NotPrivate::Owner(meta.uid()));
    }

    if meta.mode() & 0o022 != 0 {
        return Err(NotPrivate::Writable(meta.mode() & 0o7777));
    }

    Ok(())
}
