//! The AUR part of `update --aur` on the command line: the outdated AUR
//! packages from paru, their build orders read for the split path, then the
//! steps the upgrade takes.

#[cfg(test)]
mod tests;

use toolbox_core::Snapshot;
use toolbox_core::domains::InstalledPackages;
use toolbox_core::features::packages::aur::upgrade::{AurUpgrade, Start, start};
use toolbox_plan::PackageName;
use toolbox_services::aur_orders::{RouteProblem, read_every};
use toolbox_system::{AurOrderSource, AurUpdateSource, Paru, RouteSource as _, SystemRoute};

use crate::out::Settings;
use crate::{packages, root};

/// The AUR steps of an upgrade on this system, read with paru. Says so
/// when the setup takes the old paru path.
pub fn of_system(settings: Settings, snapshot: &Snapshot) -> AurUpgrade {
    packages::aur::warn_old_path(settings, snapshot);
    let paru = Paru::default();

    aur_steps(
        snapshot,
        &paru,
        &paru,
        SystemRoute::system().route(),
        root::run_counter(),
    )
}

/// The AUR steps of an upgrade of the packages `outdated` lists.
///
/// An answer that does not come back takes paru's own upgrade, which asks its
/// questions in the Console.
pub fn aur_steps(
    snapshot: &Snapshot,
    outdated: &dyn AurUpdateSource,
    orders: &dyn AurOrderSource,
    route: Result<(), RouteProblem>,
    counter: u64,
) -> AurUpgrade {
    let pending = outdated.pending_aur().ok().map(|updates| {
        updates
            .into_iter()
            .map(|update| update.name)
            .collect::<Vec<PackageName>>()
    });

    match start(snapshot, pending.as_deref()) {
        Start::Now(upgrade) => upgrade,
        Start::Read(waiting) => {
            let installed = snapshot
                .part::<InstalledPackages>()
                .map(|installed| installed.0.clone())
                .unwrap_or_default();

            let read = read_every(waiting.names(), orders, &installed, route, counter);
            waiting.plan(read.as_deref())
        }
    }
}
