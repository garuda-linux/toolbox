//! NetworkManager's system-wide DNS provider and Wi-Fi backend. Each is
//! one setting, so the item is [`Single`].

#[cfg(test)]
mod tests;

use std::sync::LazyLock;

use toolbox_plan::{DnsProvider, PackageName, PlannedAction, RootAction, WifiBackend};
pub use toolbox_state::settings::{CurrentDns, CurrentWifiBackend};

use super::{each, install};
use crate::domain::{Domain, Single, Stage, Staged, Summary};

#[expect(clippy::expect_used, reason = "a constant name, parsed by a test")]
static IWD: LazyLock<PackageName> =
    LazyLock::new(|| PackageName::new("iwd").expect("`iwd` is a package name"));

pub struct Dns;

/// Runs after the services, as the last system step: it restarts
/// NetworkManager, and steps before it may still need the network.
pub struct WifiBackends;

impl Domain for Dns {
    type Item = Single;
    type State = CurrentDns;
    type Wish = DnsProvider;

    const ID: &'static str = "dns";
    const STAGE: Stage = Stage::FILES;

    fn is_satisfied(_item: &Single, wish: &DnsProvider, state: &CurrentDns) -> bool {
        state.0 == *wish
    }

    fn plan(changes: &[(&Single, &DnsProvider)]) -> Vec<Staged> {
        each(Self::STAGE, changes, |_, provider| RootAction::SetDns {
            provider: *provider,
        })
    }

    single_key!();

    fn summary(_item: &Single, wish: &DnsProvider) -> Summary {
        Summary::of("dns", wish.as_str())
    }
}

impl Domain for WifiBackends {
    type Item = Single;
    type State = CurrentWifiBackend;
    type Wish = WifiBackend;

    const ID: &'static str = "wifi-backend";
    const STAGE: Stage = Stage::new(Stage::SERVICES.rank() + 20);

    fn is_satisfied(_item: &Single, wish: &WifiBackend, state: &CurrentWifiBackend) -> bool {
        state.0 == *wish
    }

    /// `plan` cannot see the snapshot, so iwd is always installed first:
    /// pacman's `--needed` makes that a no-op when it is there already.
    fn plan(changes: &[(&Single, &WifiBackend)]) -> Vec<Staged> {
        changes
            .iter()
            .flat_map(|(_, backend)| {
                let install = match backend {
                    WifiBackend::Iwd => Some(install(&IWD)),
                    WifiBackend::WpaSupplicant => None,
                };

                install
                    .into_iter()
                    .chain([RootAction::SetWifiBackend { backend: **backend }])
            })
            .map(PlannedAction::Root)
            .map(|action| Staged::at(Self::STAGE, action))
            .collect()
    }

    single_key!();

    fn summary(_item: &Single, wish: &WifiBackend) -> Summary {
        Summary::of("wifi-backend", wish.as_str())
    }
}
