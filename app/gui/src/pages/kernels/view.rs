#[cfg(test)]
mod tests;

use toolbox_core::domains::{DkmsBuilds, Packages};
use toolbox_core::features::kernels::{
    DkmsEntry, Kernel, KernelReleases, Listed, ModuleState, ModuleStatus, RunningKernel,
    build_release, builds_for, kernels_of, lacks_headers, modules,
};
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_gui_kit::Loadable;
use toolbox_plan::{DkmsBuild, PackageName};
use toolbox_system::ProbeError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Dkms {
    NotInstalled,
    Modules(Vec<DkmsEntry>),
}

/// What dkms said last. It is slow, so it is no part of the snapshot.
pub type DkmsReading = Loadable<Dkms>;

impl Dkms {
    pub fn entries(&self) -> &[DkmsEntry] {
        match self {
            Self::NotInstalled => &[],
            Self::Modules(entries) => entries,
        }
    }
}

#[derive(Debug)]
pub struct Listing {
    kernels: Vec<Kernel>,
    running: Option<PackageName>,
    releases: KernelReleases,
}

#[derive(Debug)]
pub struct Row<'listing> {
    pub kernel: &'listing Kernel,
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

    pub fn builds(&self, kernel: &Kernel, dkms: &DkmsReading) -> Vec<DkmsBuild> {
        builds_for(kernel, &self.releases, entries(dkms))
    }

    pub fn all_builds(&self, dkms: &DkmsReading) -> Vec<DkmsBuild> {
        self.kernels
            .iter()
            .flat_map(|kernel| self.builds(kernel, dkms))
            .collect()
    }

    pub fn rows(&self, changes: &ChangeSet, dkms: &DkmsReading) -> Vec<Row<'_>> {
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

    pub fn modules_missing(&self, changes: &ChangeSet, dkms: &DkmsReading) -> usize {
        self.all_builds(dkms)
            .iter()
            .filter(|build| changes.wish::<DkmsBuilds>(build).is_none())
            .count()
    }

    fn row<'listing>(
        &'listing self,
        kernel: &'listing Kernel,
        changes: &ChangeSet,
        dkms: &DkmsReading,
    ) -> Row<'listing> {
        let release = build_release(kernel, &self.releases);
        let modules = release.map_or_else(Vec::new, |release| {
            modules(release, entries(dkms))
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
            kernel,
            queued,
            running: self.running.as_ref() == Some(&kernel.name),
            headers_queued: changes.wish::<Packages>(&kernel.headers).copied(),
            lacks_headers: lacks_headers(kernel, changes),
            modules,
            wanted: queued.map_or_else(|| kernel.is_installed(), Toggle::is_on),
        }
    }
}

impl Row<'_> {
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

/// dkms ran but gave no usable answer. The page shows the reason as dkms
/// gave it, without the probe's own prefix.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct DkmsFailure(pub String);

/// # Errors
///
/// [`DkmsFailure`], the reason dkms failed with.
pub fn dkms_answer(answer: Result<Vec<DkmsEntry>, ProbeError>) -> Result<Dkms, DkmsFailure> {
    match answer {
        Ok(entries) => Ok(Dkms::Modules(entries)),
        Err(ProbeError::Unavailable) => Ok(Dkms::NotInstalled),
        Err(error) => {
            let reason = error.reason().unwrap_or_default();
            tracing::warn!("dkms status: {reason}");
            Err(DkmsFailure(reason))
        }
    }
}

fn entries(dkms: &DkmsReading) -> &[DkmsEntry] {
    dkms.get().map_or(&[], Dkms::entries)
}

/// What `uname -r` printed, empty when unknown.
pub fn release(snapshot: &Snapshot) -> &str {
    snapshot
        .part::<RunningKernel>()
        .map_or("", |booted| booted.release.as_str())
}
