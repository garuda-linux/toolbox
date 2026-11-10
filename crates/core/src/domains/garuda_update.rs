//! The switches of garuda-update's config file.

#[cfg(test)]
mod tests;

use toolbox_plan::{GarudaUpdateFlag, RootAction};
pub use toolbox_state::settings::GarudaUpdateConfig;

use super::each;
use crate::SummaryCode;
use crate::domain::{Domain, Stage, Staged, Summary, Toggle};

pub struct GarudaUpdateFlags;

impl Domain for GarudaUpdateFlags {
    type Item = GarudaUpdateFlag;
    type State = GarudaUpdateConfig;
    type Wish = Toggle;

    const ID: &'static str = "garuda-update-flags";
    const STAGE: Stage = Stage::FILES;

    fn is_satisfied(item: &GarudaUpdateFlag, wish: &Toggle, state: &GarudaUpdateConfig) -> bool {
        state.0.contains(item) == wish.is_on()
    }

    /// One edit per switch keeps each step's Console output readable.
    fn plan(changes: &[(&GarudaUpdateFlag, &Toggle)]) -> Vec<Staged> {
        each(Self::STAGE, changes, |flag, wish| {
            RootAction::SetGarudaUpdateFlag {
                flag: *flag,
                enabled: wish.is_on(),
            }
        })
    }

    fn key(item: &GarudaUpdateFlag) -> String {
        item.as_str().to_owned()
    }

    fn summary(item: &GarudaUpdateFlag, wish: &Toggle) -> Summary {
        Summary::toggle(
            *wish,
            SummaryCode::GarudaUpdateFlagOn,
            SummaryCode::GarudaUpdateFlagOff,
            item.as_str(),
        )
    }
}
