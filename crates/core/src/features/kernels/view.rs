#[cfg(test)]
mod tests;

use toolbox_plan::{DkmsBuild, PackageName};

use super::{
    DkmsEntry, Kernel, KernelReleases, Listed, ModuleState, ModuleStatus, RunningKernel,
    build_release, builds_for, kernels_of, lacks_headers, modules,
};
use crate::domains::{DkmsBuilds, Packages};
use crate::features::toggles::shown::Control;
use crate::{ChangeSet, Snapshot, Toggle};

#[derive(Debug)]
pub struct Listing {
    kernels: Vec<Kernel>,
    running: Option<PackageName>,
    releases: KernelReleases,
}

#[derive(Debug)]
pub struct Row {
    pub kernel: Kernel,
    pub queued: Option<Toggle>,
    pub running: bool,
    pub headers_queued: Option<Toggle>,
    pub lacks_headers: bool,
    /// Empty until dkms answered, and for a kernel without its headers.
    pub modules: Vec<Module>,
    /// Installed once the queue is applied: the queued wish, else now.
    pub wanted: bool,
}

#[derive(Debug)]
pub struct Module {
    pub status: ModuleStatus,
    pub queued: bool,
}

/// `dkms` in the methods is what dkms said last, empty until it answered.
impl Listing {
    /// `None` without the kernel packages: pacman's databases were not read.
    pub fn of(snapshot: &Snapshot) -> Option<Self> {
        let (kernels, running) = kernels_of(snapshot)?;
        let releases = snapshot
            .part::<KernelReleases>()
            .cloned()
            .unwrap_or_default();

        Some(Self {
            kernels,
            running,
            releases,
        })
    }

    pub fn kernels(&self) -> &[Kernel] {
        &self.kernels
    }

    pub fn kernel(&self, name: &PackageName) -> Option<&Kernel> {
        self.kernels.iter().find(|kernel| &kernel.name == name)
    }

    pub fn listed(&self) -> Listed<'_> {
        Listed {
            kernels: &self.kernels,
            running: self.running.as_ref().and_then(|name| self.kernel(name)),
        }
    }

    pub fn builds(&self, kernel: &Kernel, dkms: &[DkmsEntry]) -> Vec<DkmsBuild> {
        builds_for(kernel, &self.releases, dkms)
    }

    pub fn all_builds(&self, dkms: &[DkmsEntry]) -> Vec<DkmsBuild> {
        self.kernels
            .iter()
            .flat_map(|kernel| self.builds(kernel, dkms))
            .collect()
    }

    pub fn rows(&self, changes: &ChangeSet, dkms: &[DkmsEntry]) -> Vec<Row> {
        self.kernels
            .iter()
            .map(|kernel| self.row(kernel, changes, dkms))
            .collect()
    }

    pub fn headers_missing(&self, changes: &ChangeSet) -> usize {
        self.kernels
            .iter()
            .filter(|kernel| lacks_headers(kernel, changes))
            .count()
    }

    pub fn modules_missing(&self, changes: &ChangeSet, dkms: &[DkmsEntry]) -> usize {
        self.all_builds(dkms)
            .iter()
            .filter(|build| changes.wish::<DkmsBuilds>(build).is_none())
            .count()
    }

    /// The row owns its kernel, so a page keeps rows without the listing.
    fn row(&self, kernel: &Kernel, changes: &ChangeSet, dkms: &[DkmsEntry]) -> Row {
        let release = build_release(kernel, &self.releases);
        let modules = release.map_or_else(Vec::new, |release| {
            modules(release, dkms)
                .into_iter()
                .map(|status| {
                    let build = DkmsBuild {
                        module: status.module.clone(),
                        version: status.version.clone(),
                        kernel: release.clone(),
                    };

                    Module {
                        queued: changes.wish::<DkmsBuilds>(&build).is_some(),
                        status,
                    }
                })
                .collect()
        });

        let queued = changes.wish::<Packages>(&kernel.name).copied();

        Row {
            kernel: kernel.clone(),
            queued,
            running: self.running.as_ref() == Some(&kernel.name),
            headers_queued: changes.wish::<Packages>(&kernel.headers).copied(),
            lacks_headers: lacks_headers(kernel, changes),
            modules,
            wanted: queued.map_or_else(|| kernel.is_installed(), Toggle::is_on),
        }
    }
}

impl Row {
    /// The same buttons as a package of Components. A queued wish offers
    /// Take back, so a restored queue never strands the running kernel.
    pub const fn control(&self) -> Control {
        match (self.queued, self.kernel.is_installed()) {
            (Some(_), _) => Control::TakeBack,
            (None, true) => Control::Remove,
            (None, false) => Control::Install,
        }
    }

    pub fn unbuilt(&self) -> usize {
        self.unbuilt_modules().count()
    }

    pub fn builds_queued(&self) -> bool {
        self.unbuilt() > 0 && self.unbuilt_modules().all(|module| module.queued)
    }

    fn unbuilt_modules(&self) -> impl Iterator<Item = &Module> {
        self.modules
            .iter()
            .filter(|module| module.status.state != ModuleState::Installed)
    }
}

/// What `uname -r` printed, empty when unknown.
pub fn release(snapshot: &Snapshot) -> &str {
    snapshot
        .part::<RunningKernel>()
        .map_or("", |booted| booted.release.as_str())
}
