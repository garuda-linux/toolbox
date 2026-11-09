//! Packages through libalpm (the `alpm` crate): the installed ones from the
//! local database and the ones the configured sync databases offer. Only
//! reads, never syncs: the sync databases are as fresh as the last
//! `pacman -Sy`.

mod index;
mod orphans;
mod pacman_conf;
mod sync;
#[cfg(test)]
mod tests;

use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

use alpm::Alpm;
pub use index::PackageIndexSource;
pub use orphans::OrphanSource;
use pacman_conf::PacmanConf;
use sync::SyncCache;
use toolbox_plan::PackageName;
use toolbox_state::kernels::{LinuxPackages, LocalPackage};
use toolbox_state::packages::PackageGraph;
use toolbox_state::parts::{InstalledPackages, SyncPackages};

use crate::error::ProbeError;
use crate::names;

const SYSTEM_CONFIG: &str = "/etc/pacman.conf";

const LINUX: &str = "linux";

pub trait PackageSource: Send + Sync {
    /// Every installed package, repository or foreign.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without a package manager,
    /// [`ProbeError::Failed`] when it gives no answer.
    fn installed(&self) -> Result<InstalledPackages, ProbeError>;

    /// The installed packages' dependency graph.
    ///
    /// # Errors
    ///
    /// As [`PackageSource::installed`].
    fn graph(&self) -> Result<PackageGraph, ProbeError>;
}

pub trait SyncSource: Send + Sync {
    /// Every package of the configured sync databases.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without a package manager or without
    /// any synced database, [`ProbeError::Failed`] when one cannot be read.
    fn offered(&self) -> Result<SyncPackages, ProbeError>;
}

pub trait LinuxPackageSource: Send + Sync {
    /// Every offered and every installed package whose name starts with
    /// `linux`. Without a synced database only the installed ones.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without a package manager,
    /// [`ProbeError::Failed`] when a database cannot be read.
    fn linux_packages(&self) -> Result<LinuxPackages, ProbeError>;
}

/// [`PackageSource`], [`SyncSource`] and [`LinuxPackageSource`] through
/// libalpm, with the databases a `pacman.conf` names.
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
}

impl PackageSource for Libalpm {
    fn installed(&self) -> Result<InstalledPackages, ProbeError> {
        let handle = open_local(&self.conf()?)?;

        Ok(InstalledPackages(names::valid(
            handle.localdb().pkgs().iter().map(|package| package.name()),
        )))
    }

    fn graph(&self) -> Result<PackageGraph, ProbeError> {
        let handle = open_local(&self.conf()?)?;

        Ok(PackageGraph(
            handle
                .localdb()
                .pkgs()
                .iter()
                .filter_map(|package| index::local_node(package))
                .collect(),
        ))
    }
}

impl SyncSource for Libalpm {
    fn offered(&self) -> Result<SyncPackages, ProbeError> {
        Ok(self.sync.read(&self.conf()?)?.offered.clone())
    }
}

impl LinuxPackageSource for Libalpm {
    fn linux_packages(&self) -> Result<LinuxPackages, ProbeError> {
        let conf = self.conf()?;
        let handle = open_local(&conf)?;
        let local = handle
            .localdb()
            .pkgs()
            .iter()
            .filter(|package| package.name().starts_with(LINUX))
            .filter_map(|package| {
                Some(LocalPackage {
                    name: PackageName::new(package.name()).ok()?,
                    version: package.version().to_string(),
                    description: description(package),
                })
            })
            .collect();

        let sync = match self.sync.read(&conf) {
            Ok(read) => read.linux.clone(),
            Err(ProbeError::Unavailable) => Vec::new(),
            Err(error @ ProbeError::Failed(_)) => return Err(error),
        };

        Ok(LinuxPackages { sync, local })
    }
}

fn open_local(conf: &PacmanConf) -> Result<Alpm, ProbeError> {
    if !conf.db_path.join("local").is_dir() {
        return Err(ProbeError::Unavailable);
    }

    let handle = open(conf)?;

    // An unreadable database lists no packages, which would read as
    // "nothing installed".
    handle
        .localdb()
        .is_valid()
        .map_err(failed("the local database"))?;

    Ok(handle)
}

fn open(conf: &PacmanConf) -> Result<Alpm, ProbeError> {
    Alpm::new(c_path(&conf.root)?, c_path(&conf.db_path)?).map_err(failed("libalpm"))
}

/// libalpm takes C strings, and the `alpm` crate panics on an inner NUL.
fn c_path(path: &Path) -> Result<Vec<u8>, ProbeError> {
    let bytes = path.as_os_str().as_bytes();

    if bytes.contains(&0_u8) {
        return Err(ProbeError::Failed(format!(
            "{}: NUL in the path",
            path.display()
        )));
    }

    Ok(bytes.to_vec())
}

fn failed(what: &str) -> impl Fn(alpm::Error) -> ProbeError + '_ {
    move |error| ProbeError::Failed(format!("{what}: {error}"))
}

/// Whoever built the package wrote it, so it is outside text.
fn description(package: &alpm::Pkg) -> String {
    toolbox_plan::text::plain(package.desc().unwrap_or_default())
}
