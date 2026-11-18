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
            disabled: 2,
            masked: 0,
            dead: 1,
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

/// The command line knows only these two facts of a unit.
#[test]
fn the_state_choices_need_only_the_active_state() {
    let file = FileState::Other;
    let cases = [
        (Shown::Running, ActiveState::Reloading, true),
        (Shown::Running, ActiveState::Inactive, false),
        (Shown::Running, ActiveState::Failed, false),
        (Shown::Dead, ActiveState::Inactive, true),
        (Shown::Dead, ActiveState::Failed, false),
        (Shown::Dead, ActiveState::Deactivating, false),
        (Shown::Dead, ActiveState::Active, false),
        (Shown::Failed, ActiveState::Failed, true),
        (Shown::Failed, ActiveState::Active, false),
        (Shown::All, ActiveState::Other, true),
    ];

    for (shown, active, kept) in cases {
        assert_eq!(shown.keeps(active, file), kept, "{shown:?} {active:?}");
    }
}

#[test]
fn the_boot_choices_need_only_the_file_state() {
    let active = ActiveState::Inactive;
    let cases = [
        (Shown::Enabled, FileState::Enabled, true),
        (Shown::Enabled, FileState::Static, false),
        (Shown::Disabled, FileState::Disabled, true),
        (Shown::Disabled, FileState::Static, false),
        (Shown::Disabled, FileState::Masked, false),
        (Shown::Masked, FileState::Masked, true),
        (Shown::Masked, FileState::Disabled, false),
    ];

    for (shown, file, kept) in cases {
        assert_eq!(shown.keeps(active, file), kept, "{shown:?} {file:?}");
    }
}

#[test]
fn disabled_lists_what_is_off_at_boot_and_masked_what_cannot_start() {
    let mut units = units();
    units.push(info(
        "systemd-firstboot.service",
        "First Boot Wizard",
        ActiveState::Inactive,
        FileState::Masked,
    ));

    assert_eq!(
        names(&listed(&units, &only(Shown::Disabled), UnitOrder::Name)),
        ["avahi-daemon.service", "cups.socket"],
        "a static unit has no boot switch, so it is not disabled"
    );
    assert_eq!(
        names(&listed(&units, &only(Shown::Masked), UnitOrder::Name)),
        ["systemd-firstboot.service"]
    );
}

#[test]
fn dead_lists_what_is_stopped_but_not_failed() {
    assert_eq!(
        names(&listed(&units(), &only(Shown::Dead), UnitOrder::Name)),
        ["cups.socket"]
    );
}
