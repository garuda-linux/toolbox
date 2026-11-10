use toolbox_plan::{PlannedAction, TargetList, UserAction};

use super::{EnabledUserUnits, USER_TEARDOWN, UserUnits};
use crate::domain::{Domain as _, Staged, Summary, Toggle};
use crate::test_support::unit;

#[test]
fn user_units_disable_at_teardown_and_enable_in_the_user_session() {
    let enable = unit("a.service");
    let disable = unit("b.service");
    let staged = UserUnits::plan(&[(&enable, &Toggle::On), (&disable, &Toggle::Off)]);

    assert_eq!(
        staged,
        [
            Staged::at(
                USER_TEARDOWN,
                PlannedAction::User(UserAction::DisableUserUnits {
                    units: TargetList::new([disable]).unwrap(),
                }),
            ),
            Staged::at(
                UserUnits::STAGE,
                PlannedAction::User(UserAction::EnableUserUnits {
                    units: TargetList::new([enable]).unwrap(),
                }),
            ),
        ]
    );
}

#[test]
fn user_unit_changes_are_named_like_their_unit() {
    assert_eq!(UserUnits::key(&unit("a.service")), "a.service");
    assert_eq!(
        UserUnits::summary(&unit("a.service"), &Toggle::Off),
        Summary::of("user-unit-disable", "a.service")
    );
    assert!(UserUnits::is_satisfied(
        &unit("a.service"),
        &Toggle::On,
        &EnabledUserUnits(std::iter::once(unit("a.service")).collect())
    ));
}
