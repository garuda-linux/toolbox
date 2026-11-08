use toolbox_plan::PackageName;

use super::{HistoryAction, HistoryEntry, history};

/// The Lima session: `htop` installed, reinstalled, downgraded from the
/// archive and removed, with the hooks and scriptlet output between.
const LIMA_LOG: &str =
    include_str!("../../../../../system/src/status/pacman_log/fixtures/pacman.log");
/// The last lines of the log of a Garuda install in QEMU, with `kwin`
/// upgraded three times and `linux-zen` installed and removed.
const INSTALLED_LOG: &str =
    include_str!("../../../../../system/src/status/pacman_log/fixtures/installed-pacman.log");

/// One whole line of [`LIMA_LOG`], for the edits that break it.
fn htop_installed() -> &'static str {
    LIMA_LOG
        .lines()
        .find(|line| line.ends_with("] [ALPM] installed htop (3.5.3-1)"))
        .unwrap()
}

fn package(name: &str) -> PackageName {
    PackageName::new(name).unwrap_or_else(|error| panic!("{error}"))
}

fn entry(when: &str, action: HistoryAction, versions: &str) -> HistoryEntry {
    HistoryEntry {
        when: when.to_owned(),
        action,
        versions: versions.to_owned(),
    }
}

#[test]
fn the_history_lists_the_package_s_own_changes_newest_first() {
    assert_eq!(
        history(LIMA_LOG, &package("htop")),
        [
            entry(
                "2026-11-08T17:35:30+0100",
                HistoryAction::Removed,
                "3.5.2-1"
            ),
            entry(
                "2026-11-08T17:35:20+0100",
                HistoryAction::Downgraded,
                "3.5.3-1 -> 3.5.2-1"
            ),
            entry(
                "2026-11-08T17:35:18+0100",
                HistoryAction::Reinstalled,
                "3.5.3-1"
            ),
            entry(
                "2026-11-08T17:35:15+0100",
                HistoryAction::Installed,
                "3.5.3-1"
            ),
        ]
    );
}

#[test]
fn upgrades_are_listed_with_both_versions() {
    assert_eq!(
        history(INSTALLED_LOG, &package("kwin")),
        [
            entry(
                "2026-11-07T13:44:43+0100",
                HistoryAction::Upgraded,
                "6.8.2-1 -> 6.8.2-2"
            ),
            entry(
                "2026-11-02T21:36:01+0100",
                HistoryAction::Upgraded,
                "6.8.1-2 -> 6.8.2-1"
            ),
            entry(
                "2026-10-25T08:24:54+0100",
                HistoryAction::Upgraded,
                "6.8.0-1 -> 6.8.1-2"
            ),
        ]
    );
}

#[test]
fn a_package_whose_name_starts_another_is_not_mixed_in() {
    let zen = history(INSTALLED_LOG, &package("linux-zen"));
    let actions: Vec<_> = zen.iter().map(|line| line.action).collect();

    assert_eq!(
        actions,
        [HistoryAction::Removed, HistoryAction::Installed],
        "{zen:?}"
    );
    assert_eq!(
        history(INSTALLED_LOG, &package("linux-zen-headers")),
        [entry(
            "2026-11-07T13:46:26+0100",
            HistoryAction::Installed,
            "7.2.14.zen1-1"
        )]
    );
}

#[test]
fn a_cut_or_strange_line_is_left_out() {
    let line = htop_installed();

    assert_eq!(history(line, &package("htop")).len(), 1);

    let cut_front = line.get(40..).unwrap();
    let no_versions = line.replace(" (3.5.3-1)", "");
    let no_time = line.replace("[2026-11-08T17:35:15+0100] ", "");
    let cut_back = line.trim_end_matches(')');

    for broken in [cut_front, &no_versions, &no_time, cut_back] {
        assert_eq!(history(broken, &package("htop")).len(), 0, "{broken:?}");
    }
}

#[test]
fn the_time_shows_as_date_and_minute_in_the_log_s_own_zone() {
    let line = entry("2026-11-08T17:35:15+0100", HistoryAction::Installed, "1");
    let odd = entry("yesterday", HistoryAction::Installed, "1");

    assert_eq!(line.shown_when(), "2026-11-08 17:35");
    assert_eq!(odd.shown_when(), "yesterday");
}
