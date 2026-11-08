//! A mirror that no longer has the file the sync database names.
//!
//! The database is older than the mirror, so a full update (which syncs
//! it) followed by the same install fixes it.

#[cfg(test)]
mod tests;

use toolbox_plan::{PackageName, Plan, PlannedAction, RootAction, TargetList, Upgrader};

/// What pacman writes before the download error of one file.
const FAILED_RETRIEVING: &str = "failed retrieving file";
/// curl's words for an HTTP error, which pacman passes on.
const HTTP_ERROR: &str = "returned error: 404";

/// Whether `output` holds pacman's report of a package file the mirror
/// answered with 404.
pub fn mirror_missing(output: &str) -> bool {
    output
        .lines()
        .any(|line| line.contains(FAILED_RETRIEVING) && line.contains(HTTP_ERROR))
}

/// One plan, so one password: the system update that brings the
/// database level with the mirror, then the install that failed.
#[must_use = "a plan does nothing until it is run"]
pub fn retry_after_upgrade(packages: TargetList<PackageName>, upgrader: Upgrader) -> Plan {
    Plan::new([
        PlannedAction::Root(RootAction::UpgradeSystem { upgrader }),
        PlannedAction::Root(RootAction::InstallPackages { packages }),
    ])
}
