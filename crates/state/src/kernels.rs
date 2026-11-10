pub mod dkms;

use std::collections::BTreeMap;

pub use dkms::{DkmsEntry, DkmsState, ModuleState, ModuleStatus, builds, modules, parse_status};
use toolbox_plan::{KernelRelease, PackageName};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepoPackage {
    pub repo: String,
    pub name: PackageName,
    pub version: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalPackage {
    pub name: PackageName,
    pub version: String,
    pub description: String,
}

/// Every package whose name starts with `linux`: `sync` in the order
/// `pacman -Ss` lists them (repositories as in `pacman.conf`, names sorted
/// within one), `local` the installed ones.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LinuxPackages {
    pub sync: Vec<RepoPackage>,
    pub local: Vec<LocalPackage>,
}

/// The kernel that booted: what `uname -r` prints, and the package that
/// owns its modules folder while that folder still exists (an updated
/// kernel removes it until the next boot).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunningKernel {
    pub release: KernelRelease,
    pub package: Option<PackageName>,
}

/// The release each installed kernel package ships, from the `pkgbase`
/// files below `/usr/lib/modules`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KernelReleases(pub BTreeMap<PackageName, KernelRelease>);

impl crate::Part for KernelReleases {}

impl crate::Part for LinuxPackages {}

impl crate::Part for RunningKernel {}
