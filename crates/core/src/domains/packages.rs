//! Packages from the repositories, installed and removed by pacman as
//! root.

#[cfg(test)]
mod tests;

use toolbox_plan::{PackageName, PlannedAction, RootAction};
pub use toolbox_state::parts::{InstalledPackages, SyncPackages};

use super::{batch, split, staged};
use crate::domain::{Domain, Stage, Staged, Summary, Toggle};

pub struct Packages;

impl Domain for Packages {
    type Item = PackageName;
    type State = InstalledPackages;
    type Wish = Toggle;

    const ID: &'static str = "packages";
    const STAGE: Stage = Stage::PACKAGES;

    fn is_satisfied(item: &PackageName, wish: &Toggle, state: &InstalledPackages) -> bool {
        state.0.contains(item) == wish.is_on()
    }

    /// Removals first: a replacement must not conflict with what it
    /// replaces.
    fn plan(changes: &[(&PackageName, &Toggle)]) -> Vec<Staged> {
        let (install, remove) = split(changes);

        staged([
            batch(Self::STAGE, remove, |packages| {
                PlannedAction::Root(RootAction::RemovePackages { packages })
            }),
            batch(Self::STAGE, install, |packages| {
                PlannedAction::Root(RootAction::InstallPackages { packages })
            }),
        ])
    }

    named_key!();

    fn summary(item: &PackageName, wish: &Toggle) -> Summary {
        Summary::toggle(*wish, "package-install", "package-remove", item.as_str())
    }
}
