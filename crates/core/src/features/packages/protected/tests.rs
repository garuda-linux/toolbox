use std::collections::BTreeMap;

use toolbox_state::packages::{ProtectedPackages, Protection};

use super::may_remove;
use crate::Snapshot;
use crate::testing::package;

fn kept_sudo() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(ProtectedPackages(BTreeMap::from([(
        package("sudo"),
        Protection::Root,
    )])));
    snapshot
}

#[test]
fn a_protected_package_is_kept_with_its_reason() {
    assert_eq!(
        may_remove(&package("sudo"), &kept_sudo()),
        Err(Protection::Root)
    );
}

#[test]
fn other_packages_may_go() {
    assert_eq!(may_remove(&package("vlc"), &kept_sudo()), Ok(()));
}
