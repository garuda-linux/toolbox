//! The Tools tab: ping and traceroute in the Console as the user, and
//! the packages its buttons need.

use std::sync::LazyLock;

use toolbox_plan::{HopLimit, HostName, PackageName, PingCount, PingTimeout, Plan, UserAction};

use crate::domains::{InstalledPackages, Packages};
use crate::{ChangeSet, Snapshot, Toggle};

/// The hotspot tool the page offers.
#[expect(clippy::expect_used, reason = "a constant name, parsed by a test")]
pub static HOTSPOT_PACKAGE: LazyLock<PackageName> = LazyLock::new(|| {
    PackageName::new("linux-wifi-hotspot").expect("`linux-wifi-hotspot` is a package name")
});
/// What that package installs for the desktop.
pub const HOTSPOT_PROGRAM: &str = "/usr/bin/wihotspot-gui";
#[expect(clippy::expect_used, reason = "a constant name, parsed by a test")]
pub static TRACEROUTE_PACKAGE: LazyLock<PackageName> =
    LazyLock::new(|| PackageName::new("traceroute").expect("`traceroute` is a package name"));

/// Whether a button can start its program or first has to queue the
/// package.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageState {
    Installed,
    Queued,
    Missing,
    /// No probe delivered the installed packages.
    Unknown,
}

#[must_use = "a plan does nothing until it is run"]
pub fn plan_ping(host: HostName, count: PingCount, timeout: PingTimeout) -> Plan {
    Plan::user(UserAction::Ping {
        host,
        count,
        timeout,
    })
}

#[must_use = "a plan does nothing until it is run"]
pub fn plan_traceroute(host: HostName, max_hops: HopLimit) -> Plan {
    Plan::user(UserAction::Traceroute { host, max_hops })
}

pub fn package_state(
    package: &PackageName,
    snapshot: &Snapshot,
    changes: &ChangeSet,
) -> PackageState {
    let Some(installed) = snapshot.part::<InstalledPackages>() else {
        return PackageState::Unknown;
    };

    if installed.0.contains(package) {
        PackageState::Installed
    } else if changes.wish::<Packages>(package) == Some(&Toggle::On) {
        PackageState::Queued
    } else {
        PackageState::Missing
    }
}

/// Queues the installation of a missing package, or takes a queued one
/// back. Returns the state afterwards.
pub fn toggle_install(
    package: &PackageName,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> PackageState {
    match package_state(package, snapshot, changes) {
        PackageState::Missing => changes.want::<Packages>(package.clone(), Toggle::On, snapshot),
        PackageState::Queued => changes.remove::<Packages>(package),
        PackageState::Installed | PackageState::Unknown => {}
    }

    package_state(package, snapshot, changes)
}
