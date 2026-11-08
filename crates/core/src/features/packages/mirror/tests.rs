use toolbox_plan::{PackageName, Plan, PlannedAction, RootAction, TargetList, Upgrader};

use super::{mirror_missing, retry_after_upgrade};

/// What pacman 7 printed in a terminal when the mirror answered 404 for
/// `cowsay`, colours and carriage returns included.
const PACMAN_7: &str = include_str!("fixtures/pacman-404.stdout");

/// The curl error of [`PACMAN_7`], which the edits below change.
const CURL_404: &str = "The requested URL returned error: 404";

#[test]
fn a_file_the_mirror_no_longer_has_is_a_missing_mirror_file() {
    assert_eq!(include_str!("fixtures/pacman-404.code").trim(), "1");
    assert!(mirror_missing(PACMAN_7));

    // pacman 6 put the status text after the code.
    let pacman_6 = PACMAN_7.replace(CURL_404, "The requested URL returned error: 404 Not Found");

    assert!(mirror_missing(&pacman_6));
}

#[test]
fn colours_and_carriage_returns_of_the_terminal_do_not_hide_it() {
    let failed = PACMAN_7
        .split('\n')
        .find(|line| line.contains("failed retrieving file"))
        .unwrap();

    assert!(
        failed.contains("\u{1b}[1;31merror: \u{1b}[0m"),
        "{failed:?}"
    );
    assert!(failed.ends_with('\r'), "{failed:?}");
    assert!(mirror_missing(failed));
}

#[test]
fn other_download_failures_are_not_a_missing_file() {
    let offline = PACMAN_7.replace(CURL_404, "Could not resolve host: 127.0.0.1");
    let forbidden = PACMAN_7.replace(CURL_404, "The requested URL returned error: 403");

    assert!(!mirror_missing(&offline));
    assert!(!mirror_missing(&forbidden));
}

#[test]
fn a_404_outside_a_download_failure_is_not_one() {
    let renamed: String = PACMAN_7
        .lines()
        .filter(|line| !line.contains("failed retrieving file"))
        .flat_map(|line| [line.replace("cowsay", "http-404-page"), "\n".to_owned()])
        .collect();

    assert!(renamed.contains("404"));
    assert!(!mirror_missing(&renamed));
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
