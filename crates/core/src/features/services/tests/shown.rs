//! Each single-choice filter lists what its chip names, and the counts agree.

use super::super::{ActiveState, Counts, FileState, Filter, Shown, UnitOrder, counts};
use super::{info, listed, names, only, units};

#[test]
fn running_lists_what_is_neither_stopped_nor_failed() {
    let mut units = units();
    units.push(info(
        "upower.service",
        "Power Management",
        ActiveState::Activating,
        FileState::Disabled,
    ));

    assert_eq!(
        names(&listed(&units, &only(Shown::Running), UnitOrder::Name)),
        [
            "avahi-daemon.service",
            "sshd.service",
            "systemd-journald.service",
            "upower.service"
        ]
    );
}

#[test]
fn enabled_lists_what_starts_at_boot_and_failed_what_failed() {
    let units = units();

    assert_eq!(
        names(&listed(&units, &only(Shown::Enabled), UnitOrder::Name)),
        ["bluetooth.service", "sshd.service"]
    );
    assert_eq!(
        names(&listed(&units, &only(Shown::Failed), UnitOrder::Name)),
        ["bluetooth.service"]
    );
}

#[test]
fn the_counts_follow_the_search_and_match_what_each_choice_lists() {
    let units = units();

    assert_eq!(
        counts(&units, ""),
        Counts {
            running: 3,
            enabled: 2,
            failed: 1,
            all: 5,
        }
    );

    let searched = counts(&units, "service");
    for shown in Shown::EVERY {
        let filter = Filter {
            search: "service".to_owned(),
            shown,
        };

        assert_eq!(
            searched.of(shown),
            listed(&units, &filter, UnitOrder::Name).len(),
            "{shown:?}"
        );
    }

    assert_eq!(searched.all, 4, "cups.socket has no service in its texts");
}

/// The command line knows only the active state and whether a unit starts
/// at boot, and must list what the page lists.
#[test]
fn the_choices_need_only_the_active_state_and_the_boot_flag() {
    let cases = [
        (Shown::Running, ActiveState::Reloading, false, true),
        (Shown::Running, ActiveState::Inactive, true, false),
        (Shown::Running, ActiveState::Failed, true, false),
        (Shown::Enabled, ActiveState::Inactive, true, true),
        (Shown::Enabled, ActiveState::Active, false, false),
        (Shown::Failed, ActiveState::Failed, false, true),
        (Shown::Failed, ActiveState::Active, true, false),
        (Shown::All, ActiveState::Other, false, true),
    ];

    for (shown, active, at_boot, kept) in cases {
        assert_eq!(shown.keeps(active, at_boot), kept, "{shown:?} {active:?}");
    }
}
