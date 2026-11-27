//! The Kernels page: the kernels to list, the one that runs and what a
//! click queues.
//!
//! A kernel is a `linux*` package with a `-headers` package of the same
//! version in the same repository. That leaves out `linux-firmware`,
//! `linux-api-headers`, the docs and the tools.

mod edits;
mod names;
#[cfg(test)]
mod tests;
pub mod view;

use std::collections::HashMap;

use toolbox_plan::PackageName;
pub use toolbox_state::kernels::{
    DkmsEntry, DkmsState, KernelReleases, LinuxPackages, LocalPackage, ModuleState, ModuleStatus,
    RepoPackage, RunningKernel, builds, modules, parse_status,
};

pub use self::edits::{build_release, builds_for, set_builds, set_headers};
use self::names::{PREFIX, flavour_match, headers_of, is_base, priority};
use crate::domains::Packages;
use crate::features::packages::queue;
use crate::features::{RefusalText as _, Refused};
use crate::{ChangeSet, Snapshot, Toggle};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Kernel {
    pub name: PackageName,
    pub headers: PackageName,
    /// Empty for an installed kernel no repository offers.
    pub repo: String,
    /// What the repository offers, or the installed version without one.
    pub version: String,
    pub description: String,
    pub installed: Option<String>,
    pub headers_installed: bool,
}

impl Kernel {
    const fn new(
        name: PackageName,
        headers: PackageName,
        repo: String,
        version: String,
        description: String,
    ) -> Self {
        Self {
            name,
            headers,
            repo,
            version,
            description,
            installed: None,
            headers_installed: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Listed<'kernels> {
    pub kernels: &'kernels [Kernel],
    pub running: Option<&'kernels Kernel>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum Refusal {
    Running,
    /// Which kernel runs is not known, so none may go. A reboot after a
    /// kernel update names it again.
    RunningUnknown,
    /// Without any kernel the system does not boot.
    LastKernel,
    /// The rule every package removal meets, such as a protected
    /// dependency the removal would take along.
    Package(queue::Refusal),
}

impl Kernel {
    pub const fn is_installed(&self) -> bool {
        self.installed.is_some()
    }
}

/// The kernels to list: installed ones first, each group with the offered
/// ones in repository order before those no repository offers.
pub fn kernels(packages: &LinuxPackages, releases: &KernelReleases) -> Vec<Kernel> {
    let installed: HashMap<&PackageName, &LocalPackage> = packages
        .local
        .iter()
        .map(|package| (&package.name, package))
        .collect();

    let mut kernels: Vec<Kernel> = Vec::new();

    for candidate in offered(&packages.sync) {
        match kernels
            .iter_mut()
            .find(|kernel| kernel.name == candidate.name)
        {
            Some(listed) if priority(&candidate.repo) < priority(&listed.repo) => {
                *listed = candidate;
            }
            Some(_) => {}
            None => kernels.push(candidate),
        }
    }

    for package in &packages.local {
        let listed = kernels.iter().any(|kernel| kernel.name == package.name);
        let Some(headers) = headers_of(&package.name).filter(|_| is_base(&package.name) && !listed)
        else {
            continue;
        };

        if releases.0.contains_key(&package.name) || installed.contains_key(&headers) {
            kernels.push(Kernel::new(
                package.name.clone(),
                headers,
                String::new(),
                package.version.clone(),
                package.description.clone().into_string(),
            ));
        }
    }

    for kernel in &mut kernels {
        kernel.installed = installed
            .get(&kernel.name)
            .map(|package| package.version.clone());
        kernel.headers_installed = installed.contains_key(&kernel.headers);
    }

    kernels.sort_by_key(|kernel| !kernel.is_installed());
    kernels
}

/// `None` without the kernel packages: pacman's databases were not read.
pub fn kernels_of(snapshot: &Snapshot) -> Option<(Vec<Kernel>, Option<PackageName>)> {
    let releases = snapshot
        .part::<KernelReleases>()
        .cloned()
        .unwrap_or_default();

    let listed = kernels(snapshot.part::<LinuxPackages>()?, &releases);
    let booted = snapshot
        .part::<RunningKernel>()
        .and_then(|booted| running(booted, &listed))
        .map(|kernel| kernel.name.clone());

    Some((listed, booted))
}

/// The modules folder names it exactly. Once an update removed that
/// folder, the release's ending tells: `-arch` for `linux`, the flavour for
/// `linux-<flavour>`, the longest flavour first.
pub fn running<'kernels>(
    running: &RunningKernel,
    kernels: &'kernels [Kernel],
) -> Option<&'kernels Kernel> {
    let by_package = running
        .package
        .as_ref()
        .and_then(|package| kernels.iter().find(|kernel| &kernel.name == package));

    by_package.or_else(|| {
        kernels
            .iter()
            .filter(|kernel| kernel.is_installed())
            .filter_map(|kernel| {
                Some((
                    flavour_match(running.release.as_str(), &kernel.name)?,
                    kernel,
                ))
            })
            .max_by_key(|(length, _)| *length)
            .map(|(_, kernel)| kernel)
    })
}

/// Queues the kernel together with its headers, or takes that back. An
/// installed kernel without headers is removed alone, and taking its
/// removal back does not add the headers.
///
/// # Errors
///
/// [`Refusal`] when the kernel to remove runs, nobody knows which one runs,
/// it is the last one left, or a package rule keeps it or its headers.
/// The subject is the package the rule names.
pub fn toggle(
    kernel: &Kernel,
    wish: Toggle,
    listed: Listed<'_>,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refused<Refusal, PackageName>> {
    let on = wish.is_on();
    match (kernel.is_installed(), on) {
        (true, true) => unqueue(kernel, Toggle::Off, changes),
        (false, true) => {
            changes.want::<Packages>(kernel.name.clone(), Toggle::On, snapshot);
            changes.want::<Packages>(kernel.headers.clone(), Toggle::On, snapshot);
        }
        (false, false) => unqueue(kernel, Toggle::On, changes),
        (true, false) => {
            removable(kernel, listed, changes)
                .map_err(|refusal| Refused::new(refusal, kernel.name.clone()))?;
            let both = [kernel.name.clone(), kernel.headers.clone()];
            queue::protection_allows(&both, changes, snapshot).map_err(|refused| {
                Refused::new(Refusal::Package(refused.refusal), refused.subject)
            })?;
            changes.want::<Packages>(kernel.name.clone(), Toggle::Off, snapshot);
            changes.want::<Packages>(kernel.headers.clone(), Toggle::Off, snapshot);
        }
    }

    Ok(())
}

/// Drops every queued removal of a kernel the page would refuse, with
/// the removal of its headers.
///
/// Returns how many went, `None` without the kernel packages in
/// `snapshot`: nothing can be checked then and everything stays.
pub fn recheck(changes: &mut ChangeSet, snapshot: &Snapshot) -> Option<usize> {
    let packages = snapshot.part::<LinuxPackages>()?;

    let none = KernelReleases::default();
    let releases = snapshot.part::<KernelReleases>().unwrap_or(&none);
    let kernels = kernels(packages, releases);
    let listed = Listed {
        kernels: &kernels,
        running: snapshot
            .part::<RunningKernel>()
            .and_then(|booted| running(booted, &kernels)),
    };

    let mut dropped = 0;

    for kernel in kernels.iter().filter(|kernel| kernel.is_installed()) {
        if changes.wish::<Packages>(&kernel.name) != Some(&Toggle::Off) {
            continue;
        }

        if let Err(refusal) = removable(kernel, listed, changes) {
            unqueue(kernel, Toggle::Off, changes);
            tracing::warn!(
                "dropped the restored removal of {}: {}",
                kernel.name,
                refusal.code()
            );
            dropped += 1;
        }
    }

    Some(dropped)
}

/// Refuses to remove an installed kernel the Kernels page keeps.
///
/// Packages, the catalogs and the command line ask this too, so no way
/// past the Kernels page removes the running or the last kernel.
///
/// # Errors
///
/// [`Refusal`] as [`toggle`] gives it. Without the kernel packages, a
/// name of a kernel's shape is refused as [`Refusal::RunningUnknown`].
pub fn may_remove(
    package: &PackageName,
    changes: &ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    let Some((kernels, _)) = kernels_of(snapshot) else {
        let name = package.as_str();
        let is_kernel = name == PREFIX
            || name
                .strip_prefix(PREFIX)
                .is_some_and(|rest| rest.starts_with('-'));

        return if is_kernel {
            Err(Refusal::RunningUnknown)
        } else {
            Ok(())
        };
    };

    let Some(kernel) = kernels
        .iter()
        .find(|kernel| kernel.is_installed() && kernel.name == *package)
    else {
        return Ok(());
    };

    let listed = Listed {
        kernels: &kernels,
        running: snapshot
            .part::<RunningKernel>()
            .and_then(|booted| running(booted, &kernels)),
    };

    removable(kernel, listed, changes)
}

fn unqueue(kernel: &Kernel, headers_wish: Toggle, changes: &mut ChangeSet) {
    changes.remove::<Packages>(&kernel.name);

    if changes.wish::<Packages>(&kernel.headers) == Some(&headers_wish) {
        changes.remove::<Packages>(&kernel.headers);
    }
}

fn removable(kernel: &Kernel, listed: Listed<'_>, changes: &ChangeSet) -> Result<(), Refusal> {
    match listed.running {
        None => return Err(Refusal::RunningUnknown),
        Some(running) if running.name == kernel.name => return Err(Refusal::Running),
        Some(_) => {}
    }

    if !listed
        .kernels
        .iter()
        .any(|other| other.name != kernel.name && stays(other, changes))
    {
        return Err(Refusal::LastKernel);
    }

    Ok(())
}

/// Whether the kernel is installed without its headers and nothing queued
/// changes that: DKMS modules cannot be built for it.
pub fn lacks_headers(kernel: &Kernel, changes: &ChangeSet) -> bool {
    stays(kernel, changes)
        && !kernel.headers_installed
        && changes.wish::<Packages>(&kernel.headers) != Some(&Toggle::On)
}

pub fn install_missing_headers(
    kernels: &[Kernel],
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> usize {
    let missing: Vec<&Kernel> = kernels
        .iter()
        .filter(|kernel| lacks_headers(kernel, changes))
        .collect();

    for kernel in &missing {
        changes.want::<Packages>(kernel.headers.clone(), Toggle::On, snapshot);
    }

    missing.len()
}

fn stays(kernel: &Kernel, changes: &ChangeSet) -> bool {
    kernel.is_installed() && changes.wish::<Packages>(&kernel.name) != Some(&Toggle::Off)
}

fn offered(sync: &[RepoPackage]) -> Vec<Kernel> {
    let index: HashMap<(&str, &str), &RepoPackage> = sync
        .iter()
        .map(|package| ((package.repo.as_str(), package.name.as_str()), package))
        .collect();

    sync.iter()
        .filter(|package| is_base(&package.name))
        .filter_map(|package| {
            let headers = headers_of(&package.name)?;
            let pair = index.get(&(package.repo.as_str(), headers.as_str()))?;

            (pair.version == package.version).then(|| {
                Kernel::new(
                    package.name.clone(),
                    headers,
                    package.repo.clone(),
                    package.version.clone(),
                    package.description.clone().into_string(),
                )
            })
        })
        .collect()
}
