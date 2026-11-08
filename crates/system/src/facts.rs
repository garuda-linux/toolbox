//! What Home shows besides the update status: the installed packages, the
//! running kernel and its uptime, the free space on `/`, and the checks
//! for failed services, orphans and the package cache. Unlike the update
//! check these reads are quick, so they show while it still runs.

mod machine;
#[cfg(test)]
mod tests;

use std::sync::Arc;
use std::time::SystemTime;

pub use machine::{DiskSpace, MachineSource, SystemMachine};
use toolbox_plan::{PackageName, UnitName};

use crate::error::ProbeError;
use crate::packages::{Libalpm, OrphanSource, PackageSource, SyncSource};
use crate::units::{FailedUnitSource, Systemctl};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackageCount {
    pub installed: usize,
    /// Those no sync database offers: the AUR's and other foreign ones.
    /// `None` without a synced database, where every one would count.
    pub foreign: Option<usize>,
}

/// The facts as one read. Each keeps its own result, so one that fails
/// leaves the others shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    pub packages: Result<PackageCount, ProbeError>,
    /// What `uname -r` prints.
    pub kernel: Result<String, ProbeError>,
    pub booted: Result<SystemTime, ProbeError>,
    pub disk: Result<DiskSpace, ProbeError>,
    /// Bytes in pacman's package cache.
    pub package_cache: Result<u64, ProbeError>,
    pub orphans: Result<Vec<PackageName>, ProbeError>,
    pub failed_units: Result<Vec<UnitName>, ProbeError>,
}

#[derive(Clone)]
pub struct FactSources {
    pub installed: Arc<dyn PackageSource>,
    pub offered: Arc<dyn SyncSource>,
    pub orphans: Arc<dyn OrphanSource>,
    pub machine: Arc<dyn MachineSource>,
    pub units: Arc<dyn FailedUnitSource>,
}

impl FactSources {
    #[must_use = "the sources do nothing until probed"]
    pub fn system() -> Self {
        let alpm = Arc::new(Libalpm::system());

        Self {
            installed: Arc::<Libalpm>::clone(&alpm),
            offered: Arc::<Libalpm>::clone(&alpm),
            orphans: alpm,
            machine: Arc::new(SystemMachine::system()),
            units: Arc::new(Systemctl::new()),
        }
    }
}

/// Reads every fact. Blocks, so call it off the GUI thread.
pub fn probe_facts(sources: &FactSources) -> Facts {
    let _probe = tracing::debug_span!("probe facts").entered();

    Facts {
        packages: package_count(sources),
        kernel: sources.machine.kernel(),
        booted: sources.machine.booted(),
        disk: sources.machine.disk(),
        package_cache: sources.machine.package_cache(),
        orphans: sources.orphans.orphans(),
        failed_units: sources.units.failed_units(),
    }
}

fn package_count(sources: &FactSources) -> Result<PackageCount, ProbeError> {
    let installed = sources.installed.installed()?.0;
    let foreign = sources.offered.offered().ok().map(|offered| {
        installed
            .iter()
            .filter(|package| !offered.0.contains(*package))
            .count()
    });

    Ok(PackageCount {
        installed: installed.len(),
        foreign,
    })
}
