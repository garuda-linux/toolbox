//! systemd units of the system (enabled through the root helper, so a whole
//! apply needs one password) and of the user's session.

#[cfg(test)]
mod tests;

use toolbox_plan::{PlannedAction, RootAction, UnitName, UserAction};
pub use toolbox_state::parts::{EnabledUnits, EnabledUserUnits};

use super::{batch, split, staged};
use crate::SummaryCode;
use crate::domain::{Domain, Stage, Staged, Summary, Toggle};

/// The user's teardown runs before the system's, so the root steps of an
/// apply stay one batch behind one password.
pub const USER_TEARDOWN: Stage = Stage::new(Stage::TEARDOWN.rank() - 10);

/// Disabling runs at [`Stage::TEARDOWN`]: `systemctl disable` fails for a
/// unit whose package the same apply removed before it.
pub struct SystemUnits;

pub struct UserUnits;

impl Domain for SystemUnits {
    type Item = UnitName;
    type State = EnabledUnits;
    type Wish = Toggle;

    const ID: &'static str = "units";
    const STAGE: Stage = Stage::SERVICES;
    const CODES: &'static [SummaryCode] = &[SummaryCode::UnitDisable, SummaryCode::UnitEnable];

    fn is_satisfied(item: &UnitName, wish: &Toggle, state: &EnabledUnits) -> bool {
        state.0.contains(item) == wish.is_on()
    }

    fn plan(changes: &[(&UnitName, &Toggle)]) -> Vec<Staged> {
        let (enable, disable) = split(changes);

        staged([
            batch(Stage::TEARDOWN, disable, |units| {
                PlannedAction::Root(RootAction::DisableUnits { units })
            }),
            batch(Self::STAGE, enable, |units| {
                PlannedAction::Root(RootAction::EnableUnits { units })
            }),
        ])
    }

    named_key!();

    fn summary(item: &UnitName, wish: &Toggle) -> Summary {
        Summary::toggle(
            *wish,
            SummaryCode::UnitEnable,
            SummaryCode::UnitDisable,
            item.as_str(),
        )
    }
}

impl Domain for UserUnits {
    type Item = UnitName;
    type State = EnabledUserUnits;
    type Wish = Toggle;

    const ID: &'static str = "user-units";
    const STAGE: Stage = Stage::new(Stage::USER_SESSION.rank() + 1);
    const CODES: &'static [SummaryCode] =
        &[SummaryCode::UserUnitDisable, SummaryCode::UserUnitEnable];

    fn is_satisfied(item: &UnitName, wish: &Toggle, state: &EnabledUserUnits) -> bool {
        state.0.contains(item) == wish.is_on()
    }

    fn plan(changes: &[(&UnitName, &Toggle)]) -> Vec<Staged> {
        let (enable, disable) = split(changes);

        staged([
            batch(USER_TEARDOWN, disable, |units| {
                PlannedAction::User(UserAction::DisableUserUnits { units })
            }),
            batch(Self::STAGE, enable, |units| {
                PlannedAction::User(UserAction::EnableUserUnits { units })
            }),
        ])
    }

    named_key!();

    fn summary(item: &UnitName, wish: &Toggle) -> Summary {
        Summary::toggle(
            *wish,
            SummaryCode::UserUnitEnable,
            SummaryCode::UserUnitDisable,
            item.as_str(),
        )
    }
}
