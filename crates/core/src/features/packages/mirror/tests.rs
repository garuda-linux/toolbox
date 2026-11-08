use toolbox_plan::{PackageName, Plan, PlannedAction, RootAction, TargetList, Upgrader};

use super::{mirror_missing, retry_after_upgrade};

/// What pacman 7 printed when a mirror had dropped a file the stale
/// database still named.
const PACMAN_7: &str = "\
:: Retrieving packages...
 firefox-144.0-1-x86_64 downloading...
error: failed retrieving file 'firefox-144.0-1-x86_64.pkg.tar.zst' from mirror.example.org : The requested URL returned error: 404
warning: failed to retrieve some files
error: failed to commit transaction (failed to retrieve some files)
Errors occurred, no packages were upgraded.";

/// pacman 6 put the status text after the code.
const PACMAN_6: &str = "\
error: failed retrieving file 'mesa-1:24.2.3-1-x86_64.pkg.tar.zst' from geo.mirror.pkgbuild.com : The requested URL returned error: 404 Not Found";

#[test]
fn a_file_the_mirror_no_longer_has_is_a_missing_mirror_file() {
    assert!(mirror_missing(PACMAN_7));
    assert!(mirror_missing(PACMAN_6));
}

#[test]
fn colours_and_carriage_returns_of_the_terminal_do_not_hide_it() {
    let painted = "\u{1b}[1;31merror:\u{1b}[0m failed retrieving file 'a.pkg.tar.zst' from m.org : The requested URL returned error: 404\r";

    assert!(mirror_missing(painted));
}

#[test]
fn other_download_failures_are_not_a_missing_file() {
    let offline = "error: failed retrieving file 'core.db' from mirror.example.org : Could not resolve host: mirror.example.org";
    let forbidden = "error: failed retrieving file 'a.pkg.tar.zst' from m.org : The requested URL returned error: 403";

    assert!(!mirror_missing(offline));
    assert!(!mirror_missing(forbidden));
}

#[test]
fn a_404_outside_a_download_failure_is_not_one() {
    let unrelated = "(1/1) installing http-404-page\nerror: target not found: foo404";

    assert!(!mirror_missing(unrelated));
    assert!(!mirror_missing(""));
}

#[test]
fn the_retry_updates_the_system_then_installs_the_same_packages() {
    let packages = TargetList::new(
        ["firefox", "mesa"]
            .map(|name| PackageName::new(name).unwrap_or_else(|error| panic!("{name}: {error}"))),
    )
    .unwrap_or_else(|error| panic!("{error}"));

    let plan = retry_after_upgrade(packages.clone(), Upgrader::GarudaUpdate);

    assert_eq!(
        plan,
        Plan::new([
            PlannedAction::Root(RootAction::UpgradeSystem {
                upgrader: Upgrader::GarudaUpdate,
            }),
            PlannedAction::Root(RootAction::InstallPackages { packages }),
        ])
    );
}
