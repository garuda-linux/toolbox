//! Turning a switch off removes the `-git` build when only that one is
//! installed, so the package rules see that build, not the plain name.

use toolbox_plan::Protection;
use toolbox_state::packages::{InstallReason, LocalNode, PackageGraph};

use super::super::{Refusal, toggle};
use super::{catalog, snapshot};
use crate::features::Refused;
use crate::test_support::package;
use crate::{ChangeSet, Toggle};

#[test]
fn the_git_build_meets_the_package_rules() {
    let mut system = snapshot(&["docker-git", "efibootmgr"], &[], &[]);
    system.insert(PackageGraph(vec![
        LocalNode {
            name: package("docker-git"),
            depends: vec!["efibootmgr".to_owned()],
            provides: Vec::new(),
            reason: InstallReason::Explicit,
        },
        LocalNode {
            name: package("efibootmgr"),
            depends: Vec::new(),
            provides: Vec::new(),
            reason: InstallReason::Dependency,
        },
    ]));
    let mut changes = ChangeSet::default();

    assert_eq!(
        toggle(&catalog(), "docker", Toggle::Off, &system, &mut changes),
        Err(Refused::new(
            Refusal::TakesAlong(Protection::Boot),
            "efibootmgr".to_owned()
        ))
    );
    assert!(changes.is_empty());
}
