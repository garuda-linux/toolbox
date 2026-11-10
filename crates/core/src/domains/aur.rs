//! Packages built from the AUR by paru, as the user. Removing one is a
//! removal of the packages domain, as for any other installed package.

#[cfg(test)]
mod tests;

use toolbox_plan::{PackageName, PlannedAction, UserAction};

use super::{InstalledPackages, batch, staged};
use crate::SummaryCode;
use crate::domain::{Domain, Queued, Stage, Staged, Summary};

pub struct AurPackages;

impl Domain for AurPackages {
    type Item = PackageName;
    type State = InstalledPackages;
    type Wish = Queued;

    const ID: &'static str = "aur-packages";
    /// A build may need what the repository stage installs, and the DKMS
    /// stage may build a module an AUR package ships.
    const STAGE: Stage = Stage::new(300);
    const CODES: &'static [SummaryCode] = &[SummaryCode::AurInstall];

    fn is_satisfied(item: &PackageName, _wish: &Queued, state: &InstalledPackages) -> bool {
        state.0.contains(item)
    }

    fn plan(changes: &[(&PackageName, &Queued)]) -> Vec<Staged> {
        let names = changes.iter().map(|(item, _)| (*item).clone()).collect();

        staged([batch(Self::STAGE, names, |packages| {
            PlannedAction::User(UserAction::InstallAur { packages })
        })])
    }

    named_key!();

    fn summary(item: &PackageName, _wish: &Queued) -> Summary {
        Summary::of(SummaryCode::AurInstall, item.as_str())
    }
}
