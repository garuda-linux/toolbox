//! The reads of one snapshot: `pacman.conf` is read once and one libalpm
//! handle serves the local database and the sync databases alike.

use std::cell::OnceCell;
use std::path::Path;
use std::sync::Arc;

use alpm::Alpm;
use toolbox_plan::PackageName;
use toolbox_state::kernels::{LinuxPackages, LocalPackage};
use toolbox_state::packages::PackageGraph;
use toolbox_state::parts::{InstalledPackages, SyncPackages};

use super::pacman_conf::{self, PacmanConf};
use super::sync::{SyncCache, SyncRead};
use super::{LINUX, PackageScan, checked_local, description, index, open};
use crate::error::ProbeError;
use crate::names;

/// The scan of [`super::Libalpm`], for one snapshot.
pub struct LibalpmScan<'adapter> {
    sync: &'adapter SyncCache,
    conf: Result<PacmanConf, ProbeError>,
    /// Looked at before the handle opens, since libalpm creates the
    /// folder of the local database when it opens.
    local_present: bool,
    handle: Result<Alpm, ProbeError>,
    /// Read at most once per scan, failures included: a second sync
    /// registration on the same handle would fail for another reason.
    read: OnceCell<Result<Arc<SyncRead>, ProbeError>>,
}

impl<'adapter> LibalpmScan<'adapter> {
    pub fn open(config: &Path, sync: &'adapter SyncCache) -> Self {
        let conf = pacman_conf::read(config);
        let local_present = conf
            .as_ref()
            .is_ok_and(|conf| conf.db_path.join("local").is_dir());

        let handle = conf.as_ref().map_err(Clone::clone).and_then(open);

        Self {
            sync,
            conf,
            local_present,
            handle,
            read: OnceCell::new(),
        }
    }

    fn config(&self) -> Result<&PacmanConf, ProbeError> {
        self.conf.as_ref().map_err(Clone::clone)
    }

    fn handle(&self) -> Result<&Alpm, ProbeError> {
        self.handle.as_ref().map_err(Clone::clone)
    }

    fn local(&self) -> Result<&Alpm, ProbeError> {
        if !self.local_present {
            return Err(ProbeError::Unavailable);
        }

        checked_local(self.config()?, || self.handle())
    }

    fn synced(&self) -> Result<Arc<SyncRead>, ProbeError> {
        self.read
            .get_or_init(|| self.sync.read(self.config()?, || self.handle()))
            .clone()
    }
}

impl PackageScan for LibalpmScan<'_> {
    fn installed(&self) -> Result<InstalledPackages, ProbeError> {
        Ok(InstalledPackages(names::valid(
            self.local()?
                .localdb()
                .pkgs()
                .iter()
                .map(|package| package.name()),
        )))
    }

    fn graph(&self) -> Result<PackageGraph, ProbeError> {
        Ok(PackageGraph(
            self.local()?
                .localdb()
                .pkgs()
                .iter()
                .filter_map(|package| index::local_node(package))
                .collect(),
        ))
    }

    fn offered(&self) -> Result<SyncPackages, ProbeError> {
        Ok(self.synced()?.offered.clone())
    }

    fn linux_packages(&self) -> Result<LinuxPackages, ProbeError> {
        let local = self
            .local()?
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

        let sync = match self.synced() {
            Ok(read) => read.linux.clone(),
            Err(ProbeError::Unavailable) => Vec::new(),
            Err(error) => return Err(error),
        };

        Ok(LinuxPackages { sync, local })
    }
}
