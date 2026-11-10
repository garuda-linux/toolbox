//! Every page that removes packages asks the kernel guard first.

use super::super::{LinuxPackages, Refusal, RunningKernel, may_remove};
use super::{local, repo};
use crate::domains::{InstalledPackages, Packages};
use crate::test_support::{package, packages};
use crate::{ChangeSet, Snapshot, Toggle};

/// `linux` runs. `linux-lts` is installed beside it.
fn two_kernels() -> Snapshot {
    let names = ["linux", "linux-headers", "linux-lts", "linux-lts-headers"];
    let mut snapshot = Snapshot::default();

    snapshot.insert(LinuxPackages {
        sync: names
            .iter()
            .map(|name| repo("core", name, "6.14.0-4"))
            .collect(),
        local: names.iter().map(|name| local(name, "6.14.0-4")).collect(),
    });
    snapshot.insert(InstalledPackages(packages(&names)));
    snapshot.insert(RunningKernel {
        release: toolbox_plan::KernelRelease::new("6.14.0-4-arch").unwrap(),
        package: Some(package("linux")),
    });
    snapshot
}

#[test]
fn the_running_kernel_may_not_be_removed() {
    let refused = may_remove(&package("linux"), &ChangeSet::default(), &two_kernels());

    assert_eq!(refused, Err(Refusal::Running));
}

#[test]
fn a_kernel_beside_the_running_one_may_go() {
    let allowed = may_remove(&package("linux-lts"), &ChangeSet::default(), &two_kernels());

    assert_eq!(allowed, Ok(()));
}

/// `linux-zen` runs from a package removed since, so the running guard
/// keeps nothing and only the last-kernel rule is left.
#[test]
fn the_last_kernel_left_may_not_be_removed() {
    let mut snapshot = two_kernels();
    let mut listed = snapshot.part::<LinuxPackages>().unwrap().clone();
    listed
        .sync
        .push(repo("extra", "linux-zen", "6.13.8.zen1-1"));
    listed
        .sync
        .push(repo("extra", "linux-zen-headers", "6.13.8.zen1-1"));
    snapshot.insert(listed);
    snapshot.insert(RunningKernel {
        release: toolbox_plan::KernelRelease::new("6.13.8-zen1-1-zen").unwrap(),
        package: Some(package("linux-zen")),
    });
    let mut changes = ChangeSet::default();
    changes.want::<Packages>(package("linux"), Toggle::Off, &snapshot);

    let refused = may_remove(&package("linux-lts"), &changes, &snapshot);

    assert_eq!(refused, Err(Refusal::LastKernel));
}

#[test]
fn without_the_running_kernel_no_kernel_may_go() {
    let mut snapshot = two_kernels();
    snapshot.insert(RunningKernel {
        // A release no installed kernel names.
        release: toolbox_plan::KernelRelease::new("6.12.30").unwrap(),
        package: None,
    });

    let refused = may_remove(&package("linux-lts"), &ChangeSet::default(), &snapshot);

    assert_eq!(refused, Err(Refusal::RunningUnknown));
}

#[test]
fn other_packages_pass() {
    let allowed = may_remove(&package("vlc"), &ChangeSet::default(), &two_kernels());

    assert_eq!(allowed, Ok(()));
}

/// Without the kernel packages nobody can tell whether `linux-lts` is a
/// kernel, so a name of that shape waits for the read.
#[test]
fn an_unread_kernel_list_keeps_kernel_names() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(packages(&["linux-lts", "vlc"])));

    let changes = ChangeSet::default();

    assert_eq!(
        may_remove(&package("linux-lts"), &changes, &snapshot),
        Err(Refusal::RunningUnknown)
    );
    assert_eq!(may_remove(&package("vlc"), &changes, &snapshot), Ok(()));
}
