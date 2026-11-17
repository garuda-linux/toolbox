use std::collections::BTreeSet;

use toolbox_plan::PackageName;
use toolbox_state::packages::{InstallReason, LocalNode, PackageGraph};

use super::recheck;
use crate::domains::{InstalledPackages, Packages};
use crate::test_support::package;
use crate::{ChangeSet, Snapshot, Toggle};

fn node(name: &str, reason: InstallReason, depends: &[&str]) -> LocalNode {
    LocalNode {
        name: package(name),
        depends: depends.iter().map(|&dep| dep.to_owned()).collect(),
        provides: Vec::new(),
        optional: Vec::new(),
        reason,
    }
}

/// `grub-tools` alone pulled in `efibootmgr`, `app-a` and `app-b` share
/// `systemd-libs`, `gimp` takes only `gegl` along.
fn snapshot() -> Snapshot {
    let graph = vec![
        node("grub-tools", InstallReason::Explicit, &["efibootmgr"]),
        node("efibootmgr", InstallReason::Dependency, &[]),
        node("app-a", InstallReason::Explicit, &["systemd-libs"]),
        node("app-b", InstallReason::Explicit, &["systemd-libs"]),
        node("systemd-libs", InstallReason::Dependency, &[]),
        node("gimp", InstallReason::Explicit, &["gegl"]),
        node("gegl", InstallReason::Dependency, &[]),
        node("sudo", InstallReason::Explicit, &[]),
    ];

    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(
        graph.iter().map(|node| node.name.clone()).collect(),
    ));
    snapshot.insert(PackageGraph(graph));
    snapshot
}

/// Queued without a guard, as a restored file or a bus edit does.
fn removing(names: &[&str]) -> ChangeSet {
    let mut changes = ChangeSet::default();
    for name in names {
        changes.want::<Packages>(package(name), Toggle::Off, &Snapshot::default());
    }

    changes
}

fn queued(changes: &ChangeSet) -> BTreeSet<PackageName> {
    changes
        .queued::<Packages>()
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

#[test]
fn a_protected_package_leaves_the_queue() {
    let mut changes = removing(&["sudo", "gimp"]);

    assert_eq!(recheck(&mut changes, &snapshot()), Some(1));
    assert_eq!(queued(&changes), BTreeSet::from([package("gimp")]));
}

#[test]
fn a_removal_that_takes_a_protected_dependency_leaves_the_queue() {
    let mut changes = removing(&["grub-tools"]);

    assert_eq!(recheck(&mut changes, &snapshot()), Some(1));
    assert!(queued(&changes).is_empty());
}

/// Each alone frees nothing protected, both in one `pacman -Rns` free
/// `systemd-libs`, so the second goes.
#[test]
fn the_batch_counts_as_one_removal() {
    let mut changes = removing(&["app-a", "app-b"]);

    assert_eq!(recheck(&mut changes, &snapshot()), Some(1));
    assert_eq!(queued(&changes), BTreeSet::from([package("app-a")]));
}

/// An install waits for no graph, which a failed probe may never bring.
#[test]
fn an_install_is_no_removal() {
    let mut changes = ChangeSet::default();
    changes.want::<Packages>(package("sudo"), Toggle::On, &Snapshot::default());

    assert_eq!(recheck(&mut changes, &Snapshot::default()), Some(0));
    assert_eq!(queued(&changes), BTreeSet::from([package("sudo")]));
}

#[test]
fn it_waits_for_the_graph() {
    let mut changes = removing(&["grub-tools"]);
    let mut without_graph = Snapshot::default();
    without_graph.insert(InstalledPackages(BTreeSet::from([package("grub-tools")])));

    assert_eq!(recheck(&mut changes, &without_graph), None);
    assert_eq!(queued(&changes), BTreeSet::from([package("grub-tools")]));
}
