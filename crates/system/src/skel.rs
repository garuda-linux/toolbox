//! Resets application configs to Garuda's defaults by copying them from
//! `/etc/skel` into the home, as listed in the skel catalog. What a reset
//! overwrites is renamed to `<name>.bak-<YYYYMMDD-HHMMSS>` first.
//!
//! Runs as the user and blocks. The app calls it inside `spawn_blocking`.
//! Both roots are parameters, so tests use temporary folders.

pub mod shelf;
mod stamp;
#[cfg(test)]
mod tests;

use std::ffi::OsString;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::{fs, io};

use toolbox_catalog::SkelEntry;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ResetReport {
    /// The files and folders written in the home, in catalog order.
    pub copied: Vec<PathBuf>,
    pub backups: Vec<Backup>,
    pub skipped: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Backup {
    pub target: PathBuf,
    pub backup: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum SkelError {
    /// A folder on the way to a target is a link that leads out of the home
    /// or nowhere. Checked for every file before anything changes.
    #[error("{}: link leads out of the home folder", path.display())]
    LinkLeavesHome { path: PathBuf },
    #[error("{}: {error}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        error: io::Error,
    },
}

pub(crate) fn available<'specs>(
    specs: &'specs [SkelEntry],
    skel_root: &Path,
) -> Vec<&'specs SkelEntry> {
    specs
        .iter()
        .filter(|entry| {
            entry
                .files()
                .iter()
                .any(|file| fs::symlink_metadata(skel_root.join(file)).is_ok())
        })
        .collect()
}

/// Creates missing folders. An existing target is renamed to
/// `<target>.bak-<YYYYMMDD-HHMMSS>` (UTC of `now`) first. Skel files that do
/// not exist are skipped.
///
/// # Errors
///
/// [`SkelError::LinkLeavesHome`] before anything changes, when a folder on
/// the way to a target is a link out of `home`. [`SkelError::Io`] when a
/// file operation fails, which can leave the targets before it reset.
pub(crate) fn reset(
    entry: &SkelEntry,
    skel_root: &Path,
    home: &Path,
    now: SystemTime,
) -> Result<ResetReport, SkelError> {
    let real_home = fs::canonicalize(home).map_err(|error| io_error(home, error))?;
    let mut report = ResetReport::default();
    let mut present = Vec::new();

    for relative in entry.files() {
        let source = skel_root.join(relative);

        match fs::symlink_metadata(&source) {
            Ok(_) => {
                check_inside(home, &real_home, relative)?;
                present.push(relative);
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => report.skipped.push(source),
            Err(error) => return Err(io_error(&source, error)),
        }
    }

    let stamp = stamp::backup_stamp(now);

    for relative in present {
        let target = home.join(relative);

        if let Some(backup) = back_up(&target, &stamp)? {
            report.backups.push(Backup {
                target: target.clone(),
                backup,
            });
        }

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
        }

        copy(&skel_root.join(relative), &target)?;
        tracing::info!("reset {} from the skel defaults", target.display());
        report.copied.push(target);
    }

    Ok(report)
}

/// Fails when a folder on the way from `home` to `relative` is a link that
/// leads out of `real_home` or nowhere. The last component may be a link:
/// the reset renames it instead of following it.
fn check_inside(home: &Path, real_home: &Path, relative: &Path) -> Result<(), SkelError> {
    let mut path = home.to_path_buf();
    let mut folders = relative.components().peekable();

    while let Some(component) = folders.next() {
        if folders.peek().is_none() {
            break;
        }

        path.push(component);

        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                let inside = fs::canonicalize(&path).is_ok_and(|real| real.starts_with(real_home));

                if !inside {
                    return Err(SkelError::LinkLeavesHome { path });
                }
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(io_error(&path, error)),
        }
    }

    Ok(())
}

fn back_up(target: &Path, stamp: &str) -> Result<Option<PathBuf>, SkelError> {
    match fs::symlink_metadata(target) {
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_error(target, error)),
    }

    let mut base = OsString::from(target.as_os_str());

    base.push(format!(".bak-{stamp}"));

    let mut backup = PathBuf::from(&base);
    let mut counter = 0_u32;

    // A second reset within the same second must not replace the first backup.
    while fs::symlink_metadata(&backup).is_ok() {
        counter += 1;

        let mut name = base.clone();

        name.push(format!("-{counter}"));
        backup = PathBuf::from(name);
    }

    fs::rename(target, &backup).map_err(|error| io_error(target, error))?;
    tracing::info!("backed up {} to {}", target.display(), backup.display());
    Ok(Some(backup))
}

fn copy(source: &Path, target: &Path) -> Result<(), SkelError> {
    let metadata = fs::symlink_metadata(source).map_err(|error| io_error(source, error))?;
    let kind = metadata.file_type();

    if kind.is_symlink() {
        let link = fs::read_link(source).map_err(|error| io_error(source, error))?;

        symlink(link, target).map_err(|error| io_error(target, error))?;
    } else if kind.is_dir() {
        fs::create_dir_all(target).map_err(|error| io_error(target, error))?;

        for item in fs::read_dir(source).map_err(|error| io_error(source, error))? {
            let child = item.map_err(|error| io_error(source, error))?;

            copy(&child.path(), &target.join(child.file_name()))?;
        }

        fs::set_permissions(target, metadata.permissions())
            .map_err(|error| io_error(target, error))?;
    } else if metadata.is_file() {
        fs::copy(source, target).map_err(|error| io_error(target, error))?;
    } else {
        tracing::warn!(
            "skipped {}: neither file, folder nor link",
            source.display()
        );
    }

    Ok(())
}

fn io_error(path: &Path, error: io::Error) -> SkelError {
    SkelError::Io {
        path: path.to_path_buf(),
        error,
    }
}
