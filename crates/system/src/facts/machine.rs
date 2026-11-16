//! The running kernel, the boot time, the space on the root file system
//! and the size of pacman's package cache, from files a user may read.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::error::{MissingFact, ProbeError, from_io};
use crate::files;
use crate::host::boot_time;
use crate::kernels::OS_RELEASE;

/// pacman's default `CacheDir`. Another one in `pacman.conf` is rare
/// enough that the row may miss it.
const PACKAGE_CACHE: &str = "var/cache/pacman/pkg";

/// The size and the free space of a file system, in bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiskSpace {
    pub total: u64,
    /// What a user may still write, without the blocks kept for root.
    pub free: u64,
}

pub trait MachineSource: Send + Sync {
    /// What `uname -r` prints.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without `/proc`.
    fn kernel(&self) -> Result<String, ProbeError>;

    /// When the system started.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without `/proc`, a failed [`ProbeError`]
    /// when `/proc/stat` names no boot time.
    fn booted(&self) -> Result<SystemTime, ProbeError>;

    /// The space of the file system that holds `/`.
    ///
    /// # Errors
    ///
    /// A failed [`ProbeError`] when the file system gives no answer.
    fn disk(&self) -> Result<DiskSpace, ProbeError>;

    /// The bytes of the files in pacman's package cache.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without the cache folder,
    /// A failed [`ProbeError`] when it cannot be listed.
    fn package_cache(&self) -> Result<u64, ProbeError>;
}

/// [`MachineSource`] from the files below a root folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemMachine {
    root: PathBuf,
}

impl SystemMachine {
    #[must_use = "the adapter does nothing until asked"]
    pub fn system() -> Self {
        Self::below(PathBuf::from("/"))
    }

    /// Another root folder, for tests.
    #[must_use = "the adapter does nothing until asked"]
    pub const fn below(root: PathBuf) -> Self {
        Self { root }
    }
}

impl MachineSource for SystemMachine {
    fn kernel(&self) -> Result<String, ProbeError> {
        let release = files::read(&self.root.join(OS_RELEASE))?;
        let release = release.trim();

        if release.is_empty() {
            return Err(ProbeError::Missing(MissingFact::KernelRelease));
        }

        Ok(release.to_owned())
    }

    fn booted(&self) -> Result<SystemTime, ProbeError> {
        let stat = files::read(&self.root.join("proc/stat"))?;

        boot_time(&stat)
    }

    fn disk(&self) -> Result<DiskSpace, ProbeError> {
        let stat = rustix::fs::statvfs(&self.root).map_err(|error| ProbeError::Syscall {
            what: self.root.display().to_string(),
            error,
        })?;

        Ok(DiskSpace {
            total: stat.f_blocks.saturating_mul(stat.f_frsize),
            free: stat.f_bavail.saturating_mul(stat.f_frsize),
        })
    }

    fn package_cache(&self) -> Result<u64, ProbeError> {
        folder_size(&self.root.join(PACKAGE_CACHE))
    }
}

/// The bytes of the plain files right in `folder`. pacman keeps its
/// downloads there, a sub folder holds partial downloads only.
fn folder_size(folder: &Path) -> Result<u64, ProbeError> {
    let entries = fs::read_dir(folder).map_err(|error| from_io(folder.display(), error))?;

    Ok(entries
        .filter_map(Result::ok)
        .filter_map(|entry| entry.metadata().ok())
        .filter(fs::Metadata::is_file)
        .map(|metadata| metadata.len())
        .sum())
}
