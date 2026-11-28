//! Home's AUR button: paru's upgrade runs at once, while the split path reads
//! the build orders of the outdated packages off the frame first.

use toolbox_core::domains::InstalledPackages;
use toolbox_core::features::packages::aur::upgrade::{AurUpgrade, Start, start};
use toolbox_plan::Plan;

use crate::app::App;
use crate::pages::home::HomeProblem;

impl App {
    /// Home's AUR button. A click while the orders are read starts nothing.
    pub(in crate::app) fn start_aur_upgrade(&mut self) {
        if self.services.packages.reading_upgrade() {
            return;
        }

        let snapshot = self.current_snapshot();
        let outdated = self.services.home.outdated_aur();
        match start(&snapshot, outdated.as_deref()) {
            Start::Now(upgrade) => self.run_aur_upgrade(upgrade),
            Start::Read(waiting) => {
                let installed = snapshot
                    .part::<InstalledPackages>()
                    .map(|installed| installed.0.clone())
                    .unwrap_or_default();

                self.services.packages.read_upgrade(waiting, installed);
            }
        }
    }

    /// Runs the plan of a split upgrade whose orders landed in this frame.
    pub(in crate::app) fn take_aur_upgrade(&mut self) {
        if let Some(upgrade) = self.services.packages.take_upgrade() {
            self.run_aur_upgrade(upgrade);
        }
    }

    /// An upgrade with nothing outdated has no steps and runs nothing.
    fn run_aur_upgrade(&mut self, upgrade: AurUpgrade) {
        let actions = upgrade.after(Vec::new());
        if actions.is_empty() {
            return;
        }

        if self.run_at_once(Plan::new(actions)).is_none() {
            self.services.home.set_problem(Some(HomeProblem::Busy));
        }
    }
}
