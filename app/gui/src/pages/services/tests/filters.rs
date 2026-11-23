use egui::accesskit::{Role, Toggled};
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::features::services::{ActiveState, FileState, Shown, UnitInfo};

use super::{PageState, ServicesPage, loaded, sshd, unit};

fn failed() -> UnitInfo {
    UnitInfo {
        active: ActiveState::Failed,
        sub_state: "failed".to_owned(),
        ..unit("bluetooth.service", "Bluetooth", FileState::Enabled)
    }
}

fn three() -> egui_kittest::Harness<'static, PageState> {
    let stopped = unit("cups.service", "CUPS", FileState::Disabled);
    let mut harness = loaded(vec![sshd(), failed(), stopped]).harness(ServicesPage::default());
    harness.run();
    harness
}

fn picked(harness: &egui_kittest::Harness<'static, PageState>, chip: &str) -> bool {
    harness
        .get_by_role_and_label(Role::Button, chip)
        .accesskit_node()
        .toggled()
        == Some(Toggled::True)
}

#[test]
fn seven_chips_count_their_units_and_running_is_picked_at_first() {
    let harness = three();

    assert!(picked(&harness, "Running 1"));
    let others = [
        "Enabled 2",
        "Disabled 1",
        "Masked 0",
        "Dead 1",
        "Failed 1",
        "All 3",
    ];

    for chip in others {
        assert!(!picked(&harness, chip), "{chip}");
    }
}

#[test]
fn one_chip_is_picked_at_a_time_and_lists_its_units() {
    let cases = [
        (
            "Failed 1",
            Shown::Failed,
            "Bluetooth",
            ["CUPS", "OpenSSH Daemon"],
        ),
        (
            "Dead 1",
            Shown::Dead,
            "CUPS",
            ["Bluetooth", "OpenSSH Daemon"],
        ),
    ];

    for (chip, shown, listed, hidden) in cases {
        let mut harness = three();

        harness.get_by_role_and_label(Role::Button, chip).click();
        harness.run();

        assert_eq!(harness.state().page.filter.shown, shown);
        assert!(!picked(&harness, "Running 1"), "{chip}");
        assert!(harness.query_by_label_contains(listed).is_some(), "{chip}");
        for name in hidden {
            assert!(
                harness.query_by_label_contains(name).is_none(),
                "{chip}: {name}"
            );
        }
    }
}

#[test]
fn the_counts_follow_the_search() {
    let mut harness = three();

    "cups".clone_into(&mut harness.state_mut().page.filter.search);
    harness.run();

    for chip in ["Running 0", "Enabled 0", "Failed 0", "All 1"] {
        assert!(
            harness
                .query_by_role_and_label(Role::Button, chip)
                .is_some(),
            "{chip}"
        );
    }
}

/// A search for a unit the picked chip hides offers the units it would
/// find, so "bluetooth" never ends in a bare "No service matches".
#[test]
fn an_empty_list_offers_every_match_and_the_button_picks_all() {
    let mut harness = three();

    "cups".clone_into(&mut harness.state_mut().page.filter.search);
    harness.run();

    assert!(harness.query_by_label("No service matches.").is_some());
    harness
        .get_by_role_and_label(Role::Button, "Show the One Match")
        .click();
    harness.run();

    assert_eq!(harness.state().page.filter.shown, Shown::All);
    assert_eq!(
        harness.state().page.filter.search,
        "cups",
        "the search stays"
    );
    assert!(harness.query_by_label_contains("CUPS").is_some());
}

#[test]
fn nothing_found_at_all_offers_no_button() {
    let mut harness = three();

    "nothing-like-this".clone_into(&mut harness.state_mut().page.filter.search);
    harness.run();

    assert!(
        harness
            .query_by_label("Change the search or the filters to see more services.")
            .is_some()
    );
    assert!(harness.query_by_label_contains("Show All").is_none());
    assert!(harness.query_by_label("Show the One Match").is_none());
}

#[test]
fn the_masked_chip_lists_the_units_that_cannot_start() {
    let masked = unit("systemd-firstboot.service", "First Boot", FileState::Masked);
    let mut harness = loaded(vec![sshd(), failed(), masked]).harness(ServicesPage::default());

    harness.run();

    harness
        .get_by_role_and_label(Role::Button, "Masked 1")
        .click();
    harness.run();

    assert_eq!(harness.state().page.filter.shown, Shown::Masked);
    assert!(harness.query_by_label_contains("First Boot").is_some());
    assert!(harness.query_by_label_contains("OpenSSH Daemon").is_none());
}
