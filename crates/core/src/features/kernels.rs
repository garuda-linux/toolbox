//! The Kernels page: the kernels to list, the one that runs and what a
//! click queues.
//!
//! A kernel is a package whose name starts with `linux` and that has a
//! `-headers` package of the same version in the same repository. That
//! rule leaves out `linux-firmware`, `linux-api-headers`, the docs and the
//! tools without naming them.

mod edits;
#[cfg(test)]
mod tests;

use std::collections::HashMap;

use toolbox_plan::PackageName;
pub use toolbox_state::kernels::{
    DkmsEntry, DkmsState, KernelReleases, LinuxPackages, LocalPackage, ModuleState, ModuleStatus,
    RepoPackage, RunningKernel, builds, modules, parse_status,
};

pub use self::edits::{build_release, builds_for, set_builds, set_headers};
use crate::domains::Packages;
use crate::{ChangeSet, Snapshot, Toggle};

const PREFIX: &str = "linux";
const HEADERS: &str = "-headers";

/// The repository pacman takes a kernel from when several offer it, on a
/// default Garuda installation. Repositories not listed come last, in the
/// order of `pacman.conf`.
const REPO_PRIORITY: [&str; 4] = ["garuda", "core", "extra", "chaotic-aur"];

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
}

impl Kernel {
    pub const fn is_installed(&self) -> bool {
        self.installed.is_some()
    }
}

impl Refusal {
    pub fn as_str(self) -> &'static str {
        self.into()
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
            kernels.push(Kernel {
                name: package.name.clone(),
                headers,
                repo: String::new(),
                version: package.version.clone(),
                description: package.description.clone(),
                installed: None,
                headers_installed: false,
            });
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
/// or it is the last one left.
pub fn toggle(
    kernel: &Kernel,
    wish: Toggle,
    listed: Listed<'_>,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    let on = wish.is_on();
    match (kernel.is_installed(), on) {
        (true, true) => unqueue(kernel, Toggle::Off, changes),
        (false, true) => {
            changes.want::<Packages>(kernel.name.clone(), Toggle::On, snapshot);
            changes.want::<Packages>(kernel.headers.clone(), Toggle::On, snapshot);
        }
        (false, false) => unqueue(kernel, Toggle::On, changes),
        (true, false) => {
            removable(kernel, listed, changes)?;
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
                refusal.as_str()
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
        return if name == "linux" || name.starts_with("linux-") {
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

            (pair.version == package.version).then(|| Kernel {
                name: package.name.clone(),
                headers,
                repo: package.repo.clone(),
                version: package.version.clone(),
                description: package.description.clone(),
                installed: None,
                headers_installed: false,
            })
        })
        .collect()
}

fn is_base(name: &PackageName) -> bool {
    name.as_str().starts_with(PREFIX) && !name.as_str().ends_with(HEADERS)
}

fn headers_of(name: &PackageName) -> Option<PackageName> {
    PackageName::new(format!("{name}{HEADERS}")).ok()
}

fn priority(repo: &str) -> usize {
    REPO_PRIORITY
        .iter()
        .position(|known| *known == repo)
        .unwrap_or(REPO_PRIORITY.len())
}

fn flavour_match(release: &str, name: &PackageName) -> Option<usize> {
    let flavour = name
        .as_str()
        .strip_prefix(PREFIX)
        .and_then(|rest| rest.strip_prefix('-'));

    flavour.map_or_else(
        || (name.as_str() == PREFIX && release.contains("-arch")).then_some(0),
        |flavour| {
            release
                .ends_with(&format!("-{flavour}"))
                .then_some(flavour.len())
        },
    )
}
