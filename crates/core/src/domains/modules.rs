//! Kernel modules the app keeps from loading by themselves, through its
//! own modprobe file. Blacklists in other files are shown on the Network
//! page and never changed.

#[cfg(test)]
mod tests;

use toolbox_plan::{KernelModule, RootAction};
pub use toolbox_state::parts::OwnBlacklist;

use super::each;
use crate::SummaryCode;
use crate::domain::{Domain, Stage, Staged, Summary, Toggle};

pub struct ModuleBlacklist;

impl Domain for ModuleBlacklist {
    type Item = KernelModule;
    type State = OwnBlacklist;
    type Wish = Toggle;

    const ID: &'static str = "module-blacklist";
    const STAGE: Stage = Stage::FILES;

    fn is_satisfied(item: &KernelModule, wish: &Toggle, state: &OwnBlacklist) -> bool {
        state.0.contains(item) == wish.is_on()
    }

    /// One edit per module keeps each step's Console output readable.
    fn plan(changes: &[(&KernelModule, &Toggle)]) -> Vec<Staged> {
        each(Self::STAGE, changes, |module, wish| {
            RootAction::SetModuleBlacklisted {
                module: module.clone(),
                blacklisted: wish.is_on(),
            }
        })
    }

    named_key!();

    fn summary(item: &KernelModule, wish: &Toggle) -> Summary {
        Summary::toggle(
            *wish,
            SummaryCode::DriverBlacklist,
            SummaryCode::DriverAllow,
            item.as_str(),
        )
    }
}
