//! A kernel removal meets the rules of every package removal too.

use toolbox_plan::Protection;
use toolbox_state::packages::{InstallReason, LocalNode, PackageGraph};

use super::super::{Listed, Refusal, toggle};
use super::{named, sample};
use crate::features::Refused;
use crate::features::packages::queue;
use crate::test_support::package;
use crate::{ChangeSet, Toggle};

fn node(name: &str, depends: &[&str], reason: InstallReason) -> LocalNode {
    LocalNode {
        name: package(name),
        depends: depends.iter().map(|&dep| dep.to_owned()).collect(),
        provides: Vec::new(),
        optional: Vec::new(),
        reason,
    }
}

/// `linux-lts` alone pulled in `efibootmgr`.
#[test]
fn a_kernel_that_takes_a_protected_package_along_stays() {
    let (kernels, mut snapshot) = sample();
    snapshot.insert(PackageGraph(vec![
        node("linux", &[], InstallReason::Explicit),
        node("linux-headers", &[], InstallReason::Explicit),
        node("linux-lts", &["efibootmgr"], InstallReason::Explicit),
        node("efibootmgr", &[], InstallReason::Dependency),
    ]));
    let listed = Listed {
        kernels: &kernels,
        running: Some(named(&kernels, "linux")),
    };

    let mut changes = ChangeSet::default();

    assert_eq!(
        toggle(
            named(&kernels, "linux-lts"),
            Toggle::Off,
            listed,
            &mut changes,
            &snapshot
        ),
        Err(Refused::new(
            Refusal::Package(queue::Refusal::TakesAlong(Protection::Boot)),
            package("efibootmgr")
        ))
    );
    assert!(changes.is_empty());
}
