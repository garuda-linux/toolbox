//! Packages through libalpm (the `alpm` crate): the installed ones from the
//! local database and the ones the configured sync databases offer. Only
//! reads, never syncs: the sync databases are as fresh as the last
//! `pacman -Sy`.

mod index;
mod orphans;
mod pacman_conf;
mod scan;
mod sync;
#[cfg(test)]
mod tests;

use std::borrow::Borrow;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

use alpm::Alpm;
pub use index::PackageIndexSource;
pub use orphans::OrphanSource;
use pacman_conf::PacmanConf;
use scan::LibalpmScan;
use sync::SyncCache;
use toolbox_state::kernels::LinuxPackages;
use toolbox_state::packages::PackageGraph;
use toolbox_state::parts::{InstalledPackages, SyncPackages};
use toolbox_text::OutsideText;

use crate::error::{ProbeError, Subject};

const SYSTEM_CONFIG: &str = "/etc/pacman.conf";

const LINUX: &str = "linux";

pub trait PackageSource: Send + Sync {
    /// The reads of one snapshot. The scan reads the configuration and
    /// opens the package manager once, however many of its reads are asked.
    fn scan(&self) -> Box<dyn PackageScan + '_>;

    /// [`PackageScan::installed`] of a scan of its own.
    ///
    /// # Errors
    ///
    /// As [`PackageScan::installed`].
    fn installed(&self) -> Result<InstalledPackages, ProbeError> {
        self.scan().installed()
    }

    /// [`PackageScan::graph`] of a scan of its own.
    ///
    /// # Errors
    ///
    /// As [`PackageScan::installed`].
    fn graph(&self) -> Result<PackageGraph, ProbeError> {
        self.scan().graph()
    }
}

/// The package reads of one snapshot, each answered from the same
/// configuration and the same handle.
pub trait PackageScan {
    /// Every installed package, repository or foreign.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without a package manager,
    /// A failed [`ProbeError`] when it gives no answer.
    fn installed(&self) -> Result<InstalledPackages, ProbeError>;

    /// The installed packages' dependency graph.
    ///
    /// # Errors
    ///
    /// As [`PackageScan::installed`].
    fn graph(&self) -> Result<PackageGraph, ProbeError>;

    /// Every package of the configured sync databases.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without a package manager or without
    /// any synced database, a failed [`ProbeError`] when one cannot be read.
    fn offered(&self) -> Result<SyncPackages, ProbeError>;

    /// Every offered and every installed package whose name starts with
    /// `linux`. Without a synced database only the installed ones.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without a package manager,
    /// A failed [`ProbeError`] when a database cannot be read.
    fn linux_packages(&self) -> Result<LinuxPackages, ProbeError>;
}

pub trait SyncSource: Send + Sync {
    /// Every package of the configured sync databases.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without a package manager or without
    /// any synced database, a failed [`ProbeError`] when one cannot be read.
    fn offered(&self) -> Result<SyncPackages, ProbeError>;
}

/// [`PackageSource`] and [`SyncSource`] through libalpm, with the databases
/// a `pacman.conf` names.
///
/// Reading the sync databases means unpacking them whole, so one read
/// serves every question until `pacman -Sy` replaces a database file.
#[derive(Debug)]
pub struct Libalpm {
    config: PathBuf,
    sync: SyncCache,
}

impl Libalpm {
    #[must_use = "the adapter does nothing until asked"]
    pub fn system() -> Self {
        Self::with_config(SYSTEM_CONFIG)
    }

    /// The databases another `pacman.conf` names, such as a captured
    /// local database in the GUI tests. A release reads only the system's.
    #[cfg(feature = "testing")]
    #[must_use = "the adapter does nothing until asked"]
    pub fn with_config(config: impl Into<PathBuf>) -> Self {
        Self::reading(config.into())
    }

    #[cfg(not(feature = "testing"))]
    #[must_use = "the adapter does nothing until asked"]
    pub(crate) fn with_config(config: impl Into<PathBuf>) -> Self {
        Self::reading(config.into())
    }

    fn reading(config: PathBuf) -> Self {
        Self {
            config,
            sync: SyncCache::default(),
        }
    }

    /// Without the file this is no Arch system, even when a pacman from
    /// another distribution is installed.
    fn conf(&self) -> Result<PacmanConf, ProbeError> {
        pacman_conf::read(&self.config)
    }

    /// The kernel packages, as [`PackageScan::linux_packages`].
    ///
    /// # Errors
    ///
    /// As [`PackageScan::linux_packages`].
    pub fn linux_packages(&self) -> Result<LinuxPackages, ProbeError> {
        self.scan().linux_packages()
    }
}

impl PackageSource for Libalpm {
    fn scan(&self) -> Box<dyn PackageScan + '_> {
        Box::new(LibalpmScan::open(&self.config, &self.sync))
    }
}

impl SyncSource for Libalpm {
    fn offered(&self) -> Result<SyncPackages, ProbeError> {
        self.scan().offered()
    }
}

fn open_local(conf: &PacmanConf) -> Result<Alpm, ProbeError> {
    checked_local(conf, || open(conf))
}

/// The handle `handle` gives, once the local database is there and reads.
/// `handle` is not called without the database folder.
fn checked_local<H>(
    conf: &PacmanConf,
    handle: impl FnOnce() -> Result<H, ProbeError>,
) -> Result<H, ProbeError>
where
    H: Borrow<Alpm>,
{
    if !conf.db_path.join("local").is_dir() {
        return Err(ProbeError::Unavailable);
    }

    let handle = handle()?;

    // An unreadable database lists no packages, which would read as
    // "nothing installed".
    handle
        .borrow()
        .localdb()
        .is_valid()
        .map_err(failed(Subject::LocalDatabase))?;

    Ok(handle)
}

fn open(conf: &PacmanConf) -> Result<Alpm, ProbeError> {
    Alpm::new(c_path(&conf.root)?, c_path(&conf.db_path)?)
        .map_err(failed(Subject::Data("libalpm".to_owned())))
}

/// libalpm takes C strings, and the `alpm` crate panics on an inner NUL.
fn c_path(path: &Path) -> Result<Vec<u8>, ProbeError> {
    let bytes = path.as_os_str().as_bytes();

    if bytes.contains(&0_u8) {
        return Err(ProbeError::NulInPath {
            path: path.to_path_buf(),
        });
    }

    Ok(bytes.to_vec())
}

fn failed(what: Subject) -> impl Fn(alpm::Error) -> ProbeError {
    move |error| ProbeError::Alpm {
        what: what.clone(),
        error,
    }
}

/// Whoever built the package wrote it, so it is outside text.
fn description(package: &alpm::Pkg) -> OutsideText {
    toolbox_text::plain(package.desc().unwrap_or_default())
}

fn size(package: &alpm::Pkg) -> u64 {
    u64::try_from(package.isize()).unwrap_or_default()
}
