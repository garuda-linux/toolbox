//! hblock, the hosts file ad blocker: its timer plus the block list in
//! `/etc/hosts`.

#[cfg(test)]
mod tests;

use std::sync::LazyLock;

use toolbox_plan::{KnownPackage, PackageName, PlannedAction, RootAction};
pub use toolbox_state::settings::HblockState;

use super::install;
use crate::domain::{Domain, Single, Stage, Staged, Summary, Toggle};

static HBLOCK: LazyLock<PackageName> =
    LazyLock::new(|| const { KnownPackage::new("hblock") }.name());

/// After the services: the timer is a unit, and hblock downloads its lists
/// before a Wi-Fi backend switch restarts NetworkManager.
///
/// Switching off is teardown: it needs the timer and the program of a
/// package the same apply may remove.
pub struct Hblock;

impl Domain for Hblock {
    type Item = Single;
    type State = HblockState;
    type Wish = Toggle;

    const ID: &'static str = "hblock";
    const STAGE: Stage = Stage::new(Stage::SERVICES.rank() + 10);

    fn is_satisfied(_item: &Single, wish: &Toggle, state: &HblockState) -> bool {
        state.0 == wish.is_on()
    }

    /// `plan` cannot see the snapshot, so hblock is always installed before
    /// it is switched on: pacman's `--needed` makes that a no-op when it is
    /// there already.
    fn plan(changes: &[(&Single, &Toggle)]) -> Vec<Staged> {
        changes
            .iter()
            .flat_map(|(_, wish)| {
                let enabled = wish.is_on();
                let install = enabled.then(|| install(&HBLOCK));
                let stage = if enabled {
                    Self::STAGE
                } else {
                    Stage::TEARDOWN
                };

                install
                    .into_iter()
                    .chain([RootAction::SetHblock { enabled }])
                    .map(move |action| Staged::at(stage, PlannedAction::Root(action)))
            })
            .collect()
    }

    single_key!();

    fn summary(_item: &Single, wish: &Toggle) -> Summary {
        Summary::plain(if wish.is_on() {
            "hblock-on"
        } else {
            "hblock-off"
        })
    }
}
