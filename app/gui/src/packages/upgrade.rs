//! Home's AUR upgrade with a split path: the build orders of the outdated
//! packages are read off the frame, then App runs the plan at once.

use std::collections::BTreeSet;
use std::sync::Arc;

use toolbox_core::features::packages::aur::upgrade::{AurUpgrade, Awaiting};
use toolbox_plan::PackageName;
use toolbox_services::aur_orders::{read_every, run_counter};

use super::{Landed, Packages};

/// Where Home's AUR upgrade stands.
#[derive(Debug, Default)]
pub(super) enum UpgradeRead {
    #[default]
    Idle,
    /// The build orders of its packages are being read.
    Reading,
    /// The plan, until App runs it.
    Planned(AurUpgrade),
}

impl Packages {
    /// Reads the build orders `waiting` needs, on a blocking thread. The plan
    /// lands in [`Packages::take_upgrade`], paru's own when an order does not
    /// come back. Without a runtime nothing can be read, so paru's plan is
    /// ready at once.
    pub fn read_upgrade(&mut self, waiting: Awaiting, installed: BTreeSet<PackageName>) {
        let Some(jobs) = &self.jobs else {
            self.upgrade = UpgradeRead::Planned(waiting.plan(None));
            return;
        };

        self.upgrade = UpgradeRead::Reading;
        let source = Arc::clone(&jobs.setup.orders);
        let route = Arc::clone(&jobs.setup.route);
        let fallback = waiting.clone();
        jobs.lane.blocking(
            "aur-upgrade",
            move || {
                let orders = read_every(
                    waiting.names(),
                    source.as_ref(),
                    &installed,
                    route.route(),
                    run_counter(),
                );

                waiting.plan(orders.as_deref())
            },
            move |read| Landed::Upgrade(read.unwrap_or_else(|_| fallback.plan(None))),
        );
    }

    /// Whether an AUR upgrade of Home reads build orders now.
    pub const fn reading_upgrade(&self) -> bool {
        matches!(self.upgrade, UpgradeRead::Reading)
    }

    /// The plan of an AUR upgrade whose orders were read, once.
    pub fn take_upgrade(&mut self) -> Option<AurUpgrade> {
        match std::mem::take(&mut self.upgrade) {
            UpgradeRead::Planned(plan) => Some(plan),
            UpgradeRead::Reading => {
                self.upgrade = UpgradeRead::Reading;
                None
            }
            UpgradeRead::Idle => None,
        }
    }
}
