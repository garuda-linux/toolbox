//! The Repairs tab: the question before a repair that wipes, and the repairs
//! a missing program greys out.

use std::path::Path;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;

use super::{disabled, open_tab, page, programs, sent, with};
use crate::command::{Command, MaintenanceCommand};
use crate::maintenance::Maintenance;
use crate::programs::Programs;

#[test]
fn a_repair_that_wipes_asks_first_and_runs_on_its_danger_button() {
    let mut harness = page(with(programs(), Maintenance::default()));
    open_tab(&mut harness, "Repairs");

    let full_reset = harness
        .query_all_by_label("Run")
        .nth(4)
        .expect("Full reset has a Run button");

    full_reset.click();
    harness.run();

    assert_eq!(sent(&harness), [], "nothing runs before the answer");
    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Run Full reset?")
            .is_some()
    );

    harness.snapshot("maintenance/repair_question_dark");

    harness
        .get_by_role_and_label(Role::Button, "Full reset")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(MaintenanceCommand::RunNow(
            "remote-full-fix".to_owned()
        ))]
    );
    assert!(harness.query_by_role(Role::Dialog).is_none());
}

#[test]
fn cancel_leaves_a_wiping_repair_unrun() {
    let mut harness = page(with(programs(), Maintenance::default()));
    open_tab(&mut harness, "Repairs");

    harness
        .query_all_by_label("Run")
        .nth(6)
        .unwrap()
        .scroll_to_me();
    harness.run();
    harness.query_all_by_label("Run").nth(6).unwrap().click();
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(Role::Dialog, "Run Reset Snapper?")
            .is_some()
    );

    harness.get_by_label("Cancel").click();
    harness.run();

    assert_eq!(sent(&harness), []);
    assert!(harness.query_by_role(Role::Dialog).is_none());
}

#[test]
fn without_garuda_update_its_repairs_show_but_rest() {
    let programs = Programs::found(|path| path == Path::new("/usr/bin/garuda-health"));
    let mut harness = page(with(programs, Maintenance::default()));
    open_tab(&mut harness, "Repairs");

    assert!(harness.query_by_label("Reset audio").is_some());
    assert_eq!(
        disabled(&harness, "Run"),
        [false, false, true, true, true, true, true, true]
    );
}

#[test]
fn a_greyed_repair_names_the_package_it_needs() {
    let programs = Programs::found(|path| path == Path::new("/usr/bin/garuda-health"));
    let mut harness = page(with(programs, Maintenance::default()));
    open_tab(&mut harness, "Repairs");

    assert_eq!(harness.query_all_by_label("Needs garuda-update").count(), 6);
    assert!(harness.query_by_label("Needs garuda-health").is_none());
}
