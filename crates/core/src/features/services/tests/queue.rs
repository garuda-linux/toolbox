//! Each unit wish lands in the domain of its kind and scope.

use toolbox_plan::UnitScope;

use super::super::{UnitWish, queue};
use crate::domains::{SystemUnitMasks, SystemUnits, UserUnitMasks, UserUnits};
use crate::test_support::unit;
use crate::{ChangeSet, Snapshot, Toggle};

fn queued(scope: UnitScope, wish: UnitWish) -> ChangeSet {
    let mut changes = ChangeSet::default();
    queue(
        &mut changes,
        scope,
        unit("sshd.service"),
        wish,
        &Snapshot::default(),
    );
    changes
}

#[test]
fn boot_and_mask_wishes_go_to_their_scopes_domain() {
    let name = unit("sshd.service");
    let on = Some(&Toggle::On);

    assert_eq!(
        queued(UnitScope::System, UnitWish::AtBoot(Toggle::On)).wish::<SystemUnits>(&name),
        on
    );
    assert_eq!(
        queued(UnitScope::User, UnitWish::AtBoot(Toggle::On)).wish::<UserUnits>(&name),
        on
    );
    assert_eq!(
        queued(UnitScope::System, UnitWish::Masked(Toggle::On)).wish::<SystemUnitMasks>(&name),
        on
    );
    assert_eq!(
        queued(UnitScope::User, UnitWish::Masked(Toggle::On)).wish::<UserUnitMasks>(&name),
        on
    );
}
