//! Masked systemd units, of the system (through the root helper) and of
//! the user's session. `On` means masked.
//!
//! Each mask domain runs just before its units domain: a masked unit
//! cannot be enabled, so an unmask has to precede the enable of the same
//! unit in one apply. Masking is teardown and follows the disable of the
//! same unit, which systemd skips for a unit that is masked already.

#[cfg(test)]
mod tests;

use toolbox_plan::{PlannedAction, RootAction, UnitName, UserAction};
use toolbox_state::parts::{MaskedUnits, MaskedUserUnits};

use super::units::USER_TEARDOWN;
use super::{batch, split, staged};
use crate::SummaryCode;
use crate::domain::{Domain, Stage, Staged, Summary, Toggle};

const SYSTEM_MASK: Stage = Stage::new(Stage::TEARDOWN.rank() + 5);
const USER_MASK: Stage = Stage::new(USER_TEARDOWN.rank() + 5);

pub struct SystemUnitMasks;

pub struct UserUnitMasks;

impl Domain for SystemUnitMasks {
    type Item = UnitName;
    type State = MaskedUnits;
    type Wish = Toggle;

    const ID: &'static str = "unit-masks";
    const STAGE: Stage = Stage::new(Stage::SERVICES.rank() - 10);
    const CODES: &'static [SummaryCode] = &[SummaryCode::UnitMask, SummaryCode::UnitUnmask];

    fn is_satisfied(item: &UnitName, wish: &Toggle, state: &MaskedUnits) -> bool {
        state.0.contains(item) == wish.is_on()
    }

    fn plan(changes: &[(&UnitName, &Toggle)]) -> Vec<Staged> {
        let (mask, unmask) = split(changes);

        staged([
            batch(SYSTEM_MASK, mask, |units| {
                PlannedAction::Root(RootAction::MaskUnits { units })
            }),
            batch(Self::STAGE, unmask, |units| {
                PlannedAction::Root(RootAction::UnmaskUnits { units })
            }),
        ])
    }

    named_key!();

    fn summary(item: &UnitName, wish: &Toggle) -> Summary {
        Summary::toggle(
            *wish,
            SummaryCode::UnitMask,
            SummaryCode::UnitUnmask,
            item.as_str(),
        )
    }
}

impl Domain for UserUnitMasks {
    type Item = UnitName;
    type State = MaskedUserUnits;
    type Wish = Toggle;

    const ID: &'static str = "user-unit-masks";
    const STAGE: Stage = Stage::new(Stage::USER_SESSION.rank() - 10);
    const CODES: &'static [SummaryCode] = &[SummaryCode::UserUnitMask, SummaryCode::UserUnitUnmask];

    fn is_satisfied(item: &UnitName, wish: &Toggle, state: &MaskedUserUnits) -> bool {
        state.0.contains(item) == wish.is_on()
    }

    fn plan(changes: &[(&UnitName, &Toggle)]) -> Vec<Staged> {
        let (mask, unmask) = split(changes);

        staged([
            batch(USER_MASK, mask, |units| {
                PlannedAction::User(UserAction::MaskUserUnits { units })
            }),
            batch(Self::STAGE, unmask, |units| {
                PlannedAction::User(UserAction::UnmaskUserUnits { units })
            }),
        ])
    }

    named_key!();

    fn summary(item: &UnitName, wish: &Toggle) -> Summary {
        Summary::toggle(
            *wish,
            SummaryCode::UserUnitMask,
            SummaryCode::UserUnitUnmask,
            item.as_str(),
        )
    }
}
