use std::collections::BTreeSet;

use toolbox_plan::{PackageName, Protection};
use toolbox_state::packages::{InstallReason, LocalNode, PackageGraph};

use super::{Refusal, toggle};
use crate::domains::{InstalledPackages, Packages, SyncPackages};
use crate::{ChangeSet, Snapshot, Toggle};

fn name(text: &str) -> PackageName {
    PackageName::new(text).unwrap_or_else(|error| panic!("{text}: {error}"))
}

fn explicit(text: &str, depends: &[&str]) -> LocalNode {
    LocalNode {
        name: name(text),
        depends: depends.iter().map(|&dep| dep.to_owned()).collect(),
        provides: Vec::new(),
        reason: InstallReason::Explicit,
    }
}

/// gimp installed, inkscape offered, paru installed from elsewhere. The
/// graph is read and takes nothing along.
fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::from([
        name("gimp"),
        name("paru"),
    ])));
    snapshot.insert(SyncPackages(BTreeSet::from([
        name("gimp"),
        name("inkscape"),
    ])));
    snapshot.insert(PackageGraph::default());
    snapshot
}

#[test]
fn an_offered_package_is_queued_for_install() {
    let mut changes = ChangeSet::default();

    assert_eq!(
        toggle(&name("inkscape"), Toggle::On, &snapshot(), &mut changes),
        Ok(())
    );
    assert_eq!(
        changes.wish::<Packages>(&name("inkscape")),
        Some(&Toggle::On)
    );
}

#[test]
fn an_installed_package_foreign_or_not_is_queued_for_removal() {
    let mut changes = ChangeSet::default();

    assert_eq!(
        toggle(&name("paru"), Toggle::Off, &snapshot(), &mut changes),
        Ok(())
    );
    assert_eq!(changes.wish::<Packages>(&name("paru")), Some(&Toggle::Off));
}

#[test]
fn a_second_click_takes_the_wish_back() {
    let snapshot = snapshot();
    let mut changes = ChangeSet::default();

    assert_eq!(
        toggle(&name("gimp"), Toggle::Off, &snapshot, &mut changes),
        Ok(())
    );
    assert_eq!(
        toggle(&name("gimp"), Toggle::On, &snapshot, &mut changes),
        Ok(())
    );
    assert_eq!(changes.queued::<Packages>().len(), 0);
}

#[test]
fn a_package_no_repository_offers_cannot_be_installed() {
    let mut changes = ChangeSet::default();

    assert_eq!(
        toggle(&name("nothing"), Toggle::On, &snapshot(), &mut changes),
        Err(Refusal::NotOffered)
    );
    assert_eq!(changes.queued::<Packages>().len(), 0);
}

#[test]
fn without_the_databases_nothing_is_queued() {
    let mut changes = ChangeSet::default();

    assert_eq!(
        toggle(
            &name("gimp"),
            Toggle::Off,
            &Snapshot::default(),
            &mut changes
        ),
        Err(Refusal::Unread)
    );
}

#[test]
fn every_refusal_has_a_code() {
    assert_eq!(
        [
            Refusal::NotOffered,
            Refusal::Unread,
            Refusal::RunningKernel,
            Refusal::RunningUnknown,
            Refusal::LastKernel,
        ]
        .map(|refusal| refusal.as_str()),
        [
            "not-offered",
            "unread",
            "running-kernel",
            "running-unknown",
            "last-kernel"
        ]
    );
}

/// The Kernels page keeps the running kernel, so Packages does too.
#[test]
fn the_running_kernel_is_not_queued_for_removal() {
    let mut snapshot = snapshot();
    let names = ["linux", "linux-headers"];
    snapshot.insert(toolbox_state::kernels::LinuxPackages {
        sync: Vec::new(),
        local: names
            .iter()
            .map(|package| toolbox_state::kernels::LocalPackage {
                name: name(package),
                version: "6.14.0-4".to_owned(),
                description: String::new(),
            })
            .collect(),
    });
    snapshot.insert(toolbox_state::kernels::RunningKernel {
        release: toolbox_plan::KernelRelease::new("6.14.0-4-arch").unwrap(),
        package: Some(name("linux")),
    });
    let mut installed = snapshot.part::<InstalledPackages>().unwrap().0.clone();
    installed.extend(names.map(name));
    snapshot.insert(InstalledPackages(installed));
    let mut changes = ChangeSet::default();

    assert_eq!(
        toggle(&name("linux"), Toggle::Off, &snapshot, &mut changes),
        Err(Refusal::RunningKernel)
    );
    assert_eq!(changes.wish::<Packages>(&name("linux")), None);
}

#[test]
fn a_protected_package_is_not_queued_for_removal() {
    let mut snapshot = snapshot();
    let mut installed = snapshot.part::<InstalledPackages>().unwrap().0.clone();
    installed.insert(name("sddm"));
    snapshot.insert(InstalledPackages(installed));
    let mut changes = ChangeSet::default();

    assert_eq!(
        toggle(&name("sddm"), Toggle::Off, &snapshot, &mut changes),
        Err(Refusal::Protected(Protection::Desktop))
    );
    assert!(changes.is_empty());
}

/// Each removal alone keeps `systemd-libs`, the queued one plus the new
/// one would take it along.
#[test]
fn a_removal_is_checked_together_with_the_queued_ones() {
    let mut snapshot = snapshot();
    snapshot.insert(InstalledPackages(BTreeSet::from([
        name("app-a"),
        name("app-b"),
        name("systemd-libs"),
    ])));
    snapshot.insert(PackageGraph(vec![
        explicit("app-a", &["systemd-libs"]),
        explicit("app-b", &["systemd-libs"]),
        LocalNode {
            reason: InstallReason::Dependency,
            ..explicit("systemd-libs", &[])
        },
    ]));
    let mut changes = ChangeSet::default();

    assert_eq!(
        toggle(&name("app-a"), Toggle::Off, &snapshot, &mut changes),
        Ok(())
    );
    assert_eq!(
        toggle(&name("app-b"), Toggle::Off, &snapshot, &mut changes),
        Err(Refusal::TakesAlong(Protection::Boot, name("systemd-libs")))
    );
    assert_eq!(changes.wish::<Packages>(&name("app-b")), None);
}

#[test]
fn without_the_dependency_graph_nothing_is_removed() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::from([name("gimp")])));
    let mut changes = ChangeSet::default();

    assert_eq!(
        toggle(&name("gimp"), Toggle::Off, &snapshot, &mut changes),
        Err(Refusal::Unread)
    );
}
