use toolbox_plan::Protection;
use toolbox_state::packages::{InstallReason, LocalNode, PackageGraph};

use super::{CascadeRefusal, may_remove, may_remove_with_cascade};
use crate::Snapshot;
use crate::test_support::package;

fn node(name: &str, reason: InstallReason, depends: &[&str]) -> LocalNode {
    LocalNode {
        name: package(name),
        depends: depends.iter().map(|&dep| dep.to_owned()).collect(),
        provides: Vec::new(),
        reason,
    }
}

/// `efibootmgr` was pulled in by `grub-tools` alone, so removing that
/// takes it along. `systemd-libs` is shared by two explicit packages.
fn graph() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(PackageGraph(vec![
        node("grub-tools", InstallReason::Explicit, &["efibootmgr"]),
        node("efibootmgr", InstallReason::Dependency, &[]),
        node("app-a", InstallReason::Explicit, &["systemd-libs"]),
        node("app-b", InstallReason::Explicit, &["systemd-libs"]),
        node("systemd-libs", InstallReason::Dependency, &[]),
        node("gimp", InstallReason::Explicit, &["gegl"]),
        node("gegl", InstallReason::Dependency, &[]),
    ]));
    snapshot
}

#[test]
fn a_protected_package_is_kept_with_its_reason() {
    assert_eq!(may_remove(&package("sudo")), Err(Protection::Root));
}

#[test]
fn other_packages_may_go() {
    assert_eq!(may_remove(&package("vlc")), Ok(()));
}

#[test]
fn a_removal_that_takes_only_free_dependencies_passes() {
    assert_eq!(
        may_remove_with_cascade(&[package("gimp")], &graph()),
        Ok(())
    );
}

#[test]
fn a_removal_that_takes_a_protected_dependency_along_is_refused() {
    assert_eq!(
        may_remove_with_cascade(&[package("grub-tools")], &graph()),
        Err(CascadeRefusal::TakesAlong(
            Protection::Boot,
            package("efibootmgr")
        ))
    );
}

/// Neither removal alone frees `systemd-libs`, both together do, as one
/// `pacman -Rns` over the whole batch.
#[test]
fn the_batch_counts_as_one_removal() {
    assert_eq!(
        may_remove_with_cascade(&[package("app-a")], &graph()),
        Ok(())
    );
    assert_eq!(
        may_remove_with_cascade(&[package("app-a"), package("app-b")], &graph()),
        Err(CascadeRefusal::TakesAlong(
            Protection::Boot,
            package("systemd-libs")
        ))
    );
}

#[test]
fn without_the_graph_no_removal_passes() {
    assert_eq!(
        may_remove_with_cascade(&[package("gimp")], &Snapshot::default()),
        Err(CascadeRefusal::Unread)
    );
}
