//! The kernels of the running system: the one that booted, the release of
//! every installed kernel package, and what dkms says about its modules.
//!
//! An Arch kernel package ships `/usr/lib/modules/<release>/pkgbase` with
//! its own name in it. That ties a package to its release without
//! guessing from version numbers, which goes wrong for kernels such as
//! `linux-zen` (package `6.13.8.zen1-1`, release `6.13.8-zen1-1-zen`).

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;

use toolbox_plan::{KernelRelease, PackageName};
use toolbox_state::kernels::{DkmsEntry, KernelReleases, RunningKernel, parse_status};

use crate::error::{ProbeError, from_io};
use crate::{command, files};

pub const OS_RELEASE: &str = "proc/sys/kernel/osrelease";
pub const MODULES: &str = "usr/lib/modules";
const PKGBASE: &str = "pkgbase";
const DKMS: &str = "/usr/bin/dkms";

pub trait KernelSource: Send + Sync {
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without `/proc`, [`ProbeError::Failed`]
    /// when the release cannot be read.
    fn running(&self) -> Result<RunningKernel, ProbeError>;

    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without `/usr/lib/modules`,
    /// [`ProbeError::Failed`] when it cannot be listed.
    fn releases(&self) -> Result<KernelReleases, ProbeError>;
}

pub trait DkmsSource: Send + Sync {
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without dkms, [`ProbeError::Failed`]
    /// when it prints nothing usable.
    fn status(&self) -> Result<Vec<DkmsEntry>, ProbeError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemKernels {
    root: PathBuf,
}

/// [`DkmsSource`] by running `dkms status` as the user: `/var/lib/dkms` is
/// world readable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dkms {
    program: OsString,
}

impl SystemKernels {
    #[must_use = "the adapter does nothing until asked"]
    pub fn system() -> Self {
        Self::new("/")
    }

    #[must_use = "the adapter does nothing until asked"]
    pub(crate) fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn package_of(&self, release: &str) -> Option<PackageName> {
        let file = self.root.join(MODULES).join(release).join(PKGBASE);
        let name = files::read_lossy(&file).ok()?;

        PackageName::new(name.trim()).ok()
    }
}

impl Dkms {
    #[must_use = "the adapter does nothing until asked"]
    pub fn system() -> Self {
        Self::with_program(DKMS)
    }

    #[must_use = "the adapter does nothing until asked"]
    pub(crate) fn with_program(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
        }
    }
}

impl KernelSource for SystemKernels {
    fn running(&self) -> Result<RunningKernel, ProbeError> {
        let text = files::read(&self.root.join(OS_RELEASE))?;

        // A release that is no single path component names no folder, and
        // no kernel prints one.
        let release = KernelRelease::new(text.trim())
            .map_err(|error| ProbeError::Failed(format!("the running kernel: {error}")))?;

        let package = self.package_of(release.as_str());

        Ok(RunningKernel { release, package })
    }

    fn releases(&self) -> Result<KernelReleases, ProbeError> {
        let folder = self.root.join(MODULES);
        let entries =
            std::fs::read_dir(&folder).map_err(|error| from_io(folder.display(), error))?;

        let mut releases: Vec<KernelRelease> = entries
            .filter_map(Result::ok)
            .filter_map(|entry| KernelRelease::new(entry.file_name().to_str()?).ok())
            .collect();

        // Two folders of one package are leftovers of an update, and the
        // newer release counts. Compared as versions: as text `6.9.10`
        // comes before `6.9.9`.
        releases.sort_by(|one, other| alpm::vercmp(one.as_str(), other.as_str()));

        let by_package: BTreeMap<PackageName, KernelRelease> = releases
            .into_iter()
            .filter_map(|release| Some((self.package_of(release.as_str())?, release)))
            .collect();

        Ok(KernelReleases(by_package))
    }
}

impl DkmsSource for Dkms {
    /// dkms exits with 0 for a broken module and reports it on stderr, so
    /// the exit code tells nothing about the modules. Its output counts
    /// whenever it holds a module.
    fn status(&self) -> Result<Vec<DkmsEntry>, ProbeError> {
        let finished = command::run(&self.program, &["status"])?;
        let entries = parse_status(&finished.stdout);

        if entries.is_empty() && !finished.status.success() {
            return Err(finished.failure());
        }

        Ok(entries)
    }
}
