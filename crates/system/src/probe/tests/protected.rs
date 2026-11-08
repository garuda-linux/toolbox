//! The protected catalog travels with every snapshot, so each way to a
//! package removal checks the same list.

use std::collections::BTreeMap;

use toolbox_plan::PackageName;
use toolbox_state::packages::{ProtectedPackages, Protection};

use super::fakes::{FakePackages, sources};
use super::{host, installed};
use crate::probe_snapshot;

#[test]
fn the_protected_packages_land_in_the_snapshot() {
    let kept = ProtectedPackages(BTreeMap::from([(
        PackageName::new("sudo").unwrap(),
        Protection::Root,
    )]));

    let sources = sources(FakePackages(Ok(installed())), host()).with_protected(kept.clone());

    let (snapshot, _issues) = probe_snapshot(&sources);

    assert_eq!(snapshot.part::<ProtectedPackages>(), Some(&kept));
}
