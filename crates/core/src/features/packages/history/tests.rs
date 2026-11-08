use toolbox_plan::PackageName;

use super::{HistoryAction, HistoryEntry, history};

/// Lines of a real `/var/log/pacman.log`, with the hooks and scriptlet
/// output between them.
const LOG: &str = "\
[2025-01-10T09:12:01+0100] [PACMAN] Running 'pacman -S firefox'
[2025-01-10T09:12:05+0100] [ALPM] transaction started
[2025-01-10T09:12:06+0100] [ALPM] installed firefox (134.0-1)
[2025-01-10T09:12:06+0100] [ALPM] installed firefox-i18n-de (134.0-1)
[2025-01-10T09:12:07+0100] [ALPM] transaction completed
[2025-02-01T18:00:00+0100] [ALPM] upgraded firefox (134.0-1 -> 135.0-1)
[2025-02-01T18:00:00+0100] [ALPM-SCRIPTLET] firefox was upgraded
[2025-02-03T08:30:00+0100] [ALPM] downgraded firefox (135.0-1 -> 134.0-1)
[2025-02-04T08:30:00+0100] [ALPM] reinstalled firefox (134.0-1)
[2025-03-01T12:00:00+0100] [ALPM] removed firefox (134.0-1)
[2025-03-01T12:00:00+0100] [PACMAN] synchronizing package lists";

fn firefox() -> PackageName {
    PackageName::new("firefox").unwrap_or_else(|error| panic!("{error}"))
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
        history(LOG, &firefox()),
        [
            entry(
                "2025-03-01T12:00:00+0100",
                HistoryAction::Removed,
                "134.0-1"
            ),
            entry(
                "2025-02-04T08:30:00+0100",
                HistoryAction::Reinstalled,
                "134.0-1"
            ),
            entry(
                "2025-02-03T08:30:00+0100",
                HistoryAction::Downgraded,
                "135.0-1 -> 134.0-1"
            ),
            entry(
                "2025-02-01T18:00:00+0100",
                HistoryAction::Upgraded,
                "134.0-1 -> 135.0-1"
            ),
            entry(
                "2025-01-10T09:12:06+0100",
                HistoryAction::Installed,
                "134.0-1"
            ),
        ]
    );
}

#[test]
fn a_package_whose_name_starts_another_is_not_mixed_in() {
    let lines = history(LOG, &firefox());

    assert!(lines.iter().all(|line| !line.versions.contains("i18n")));
    assert_eq!(
        history(
            LOG,
            &PackageName::new("firefox-i18n-de").unwrap_or_else(|error| panic!("{error}"))
        )
        .len(),
        1
    );
}

#[test]
fn a_cut_or_strange_line_is_left_out() {
    let broken =
        "fox (134.0-1)\n[2025-01-10] [ALPM] installed firefox\n[ALPM] installed firefox (1)";

    assert_eq!(history(broken, &firefox()).len(), 0);
}

#[test]
fn the_time_shows_as_date_and_minute_in_the_log_s_own_zone() {
    let line = entry("2025-01-10T09:12:06+0100", HistoryAction::Installed, "1");
    let odd = entry("yesterday", HistoryAction::Installed, "1");

    assert_eq!(line.shown_when(), "2025-01-10 09:12");
    assert_eq!(odd.shown_when(), "yesterday");
}
