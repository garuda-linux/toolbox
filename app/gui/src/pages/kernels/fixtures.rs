//! A system with three kernels for the page's tests: `linux` runs with
//! its headers, `linux-lts` lacks them, `linux-zen` is not installed.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use toolbox_core::Snapshot;
use toolbox_core::domains::InstalledPackages;
use toolbox_core::features::kernels::{
    KernelReleases, LinuxPackages, LocalPackage, RepoPackage, RunningKernel, parse_status,
};
use toolbox_plan::{DkmsBuild, DkmsModuleName, DkmsVersion, KernelRelease, PackageName};

use super::view::{Dkms, DkmsReading};

pub fn package(name: &str) -> PackageName {
    PackageName::new(name).unwrap()
}

fn repo(repo: &str, name: &str, version: &str) -> RepoPackage {
    RepoPackage {
        repo: repo.to_owned(),
        name: package(name),
        version: version.to_owned(),
        description: format!("About {name}"),
    }
}

fn local(name: &str, version: &str) -> LocalPackage {
    LocalPackage {
        name: package(name),
        version: version.to_owned(),
        description: String::new(),
    }
}

pub fn system() -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(LinuxPackages {
        sync: vec![
            repo("core", "linux", "6.14.0-4"),
            repo("core", "linux-headers", "6.14.0-4"),
            repo("core", "linux-lts", "6.12.9-1"),
            repo("core", "linux-lts-headers", "6.12.9-1"),
            repo("extra", "linux-zen", "6.13.8.zen1-1"),
            repo("extra", "linux-zen-headers", "6.13.8.zen1-1"),
        ],
        local: vec![
            local("linux", "6.14.0-3"),
            local("linux-headers", "6.14.0-3"),
            local("linux-lts", "6.12.9-1"),
        ],
    });
    snapshot.insert(InstalledPackages(BTreeSet::from([
        package("linux"),
        package("linux-headers"),
        package("linux-lts"),
    ])));
    snapshot.insert(RunningKernel {
        release: KernelRelease::new("6.14.0-arch3-1").unwrap(),
        package: Some(package("linux")),
    });
    snapshot.insert(KernelReleases(BTreeMap::from([
        (package("linux"), release("6.14.0-arch3-1")),
        (package("linux-lts"), release("6.12.9-1-lts")),
    ])));
    snapshot
}

/// `nvidia` is built for `linux`, `vboxhost` for no kernel.
pub fn dkms() -> DkmsReading {
    DkmsReading::Loaded(Arc::new(Dkms::Modules(parse_status(
        "nvidia/550.90.07, 6.14.0-arch3-1, x86_64: installed\nvboxhost/7.1.6_OSE: added\n",
    ))))
}

/// The build `vboxhost` lacks on `linux`.
pub fn vboxhost_build() -> DkmsBuild {
    DkmsBuild {
        module: DkmsModuleName::new("vboxhost").unwrap(),
        version: DkmsVersion::new("7.1.6_OSE").unwrap(),
        kernel: release("6.14.0-arch3-1"),
    }
}

fn release(name: &str) -> KernelRelease {
    KernelRelease::new(name).unwrap()
}
