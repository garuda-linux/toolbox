use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::Snapshot;

use super::super::{LocalesPage, Tab};
use crate::command::{Command, LocaleCommand};
use crate::testing::{PageFixture, failure_read_again_sends};

fn on(tab: Tab) -> LocalesPage {
    LocalesPage {
        tab,
        ..LocalesPage::default()
    }
}

#[test]
fn before_the_first_read_placeholder_rows_wait_on_both_tabs() {
    for (tab, name) in [
        (Tab::Locales, "Reading the locales…"),
        (Tab::Packs, "Reading the language packs…"),
    ] {
        let fixture = PageFixture {
            probed: false,
            ..PageFixture::default()
        };

        let mut harness = fixture.harness(on(tab));

        // Read again spins while the system is read, so the frames never settle.
        harness.run_steps(3);

        assert!(
            harness
                .query_by_role_and_label(Role::ProgressIndicator, name)
                .is_some(),
            "{tab:?}"
        );
    }
}

/// The system is read again on every refresh, not only before the first
/// reading, so Read again spins while that read runs.
#[test]
fn read_again_spins_while_the_system_is_read_again() {
    let reading = PageFixture {
        snapshot_reading: true,
        ..PageFixture::with_snapshot(Snapshot::default())
    };

    let mut harness = reading.harness(on(Tab::Locales));
    harness.run_steps(3);

    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Reading")
            .is_some()
    );

    let idle = PageFixture::with_snapshot(Snapshot::default());
    let mut idle_harness = idle.harness(on(Tab::Locales));
    idle_harness.run_steps(3);

    assert!(
        idle_harness
            .query_by_role_and_label(Role::Button, "Reading")
            .is_none()
    );
}

#[test]
fn unread_packs_offer_to_read_again() {
    let fixture = PageFixture::with_snapshot(Snapshot::default());

    assert_eq!(
        failure_read_again_sends(fixture, on(Tab::Packs)),
        [Command::from(LocaleCommand::Refresh)]
    );
}

/// Locales that are not generated from `/etc/locale.gen` are no failure,
/// so reading again would show the same.
#[test]
fn a_system_without_locale_gen_says_so_without_read_again() {
    let mut harness = PageFixture::with_snapshot(Snapshot::default()).harness(on(Tab::Locales));
    harness.run();

    assert!(
        harness
            .query_by_label("Locales are not available on this system")
            .is_some()
    );
    assert_eq!(
        harness
            .query_all_by_role_and_label(Role::Button, "Read again")
            .count(),
        1,
        "only the header's"
    );
}
