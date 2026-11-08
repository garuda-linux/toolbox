use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui::Setup;

use crate::common::launch;

pub const SETUP: &str = "Select the software you would like to install.";

#[test]
fn the_app_starts_on_home() {
    let harness = launch(Setup::default());

    assert!(harness.query_by_label("Welcome to Garuda Linux").is_some());
}

#[test]
fn a_sidebar_entry_opens_its_page() {
    let mut harness = launch(Setup::default());

    harness
        .query_all_by_label("Setup Assistant")
        .next()
        .unwrap()
        .click();
    harness.run();

    assert_eq!(
        harness.query_all_by_label("Setup Assistant").count(),
        2,
        "sidebar and title"
    );
    assert!(harness.query_by_label(SETUP).is_some());
}

#[test]
fn the_sidebar_shows_its_group_headings() {
    let harness = launch(Setup::default());

    for heading in ["INSTALL", "SYSTEM", "CARE"] {
        assert!(harness.query_by_label(heading).is_some(), "{heading}");
    }
}

#[test]
fn a_collapsed_sidebar_still_names_and_opens_every_page() {
    let mut harness = launch(Setup::default());

    harness.get_by_label("Collapse sidebar").click();
    harness.run();

    assert!(harness.query_by_label("SYSTEM").is_none(), "headings hide");
    harness
        .query_all_by_label("Setup Assistant")
        .next()
        .unwrap()
        .click();
    harness.run();
    assert_eq!(harness.query_all_by_label("Setup Assistant").count(), 2);

    harness.get_by_label("Expand sidebar").click();
    harness.run();
    assert!(harness.query_by_label("SYSTEM").is_some());
}

#[test]
fn the_system_language_is_used_when_none_is_chosen() {
    let harness = launch(Setup {
        system_language: "de-DE".parse().unwrap(),
        ..Setup::default()
    });

    assert!(
        harness
            .query_by_label("Willkommen bei Garuda Linux")
            .is_some()
    );
}

#[test]
fn the_window_parts_are_named_panes() {
    let mut harness = launch(Setup::default());
    harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::J);
    harness.run();

    for pane in ["Pages", "Page", "Console"] {
        assert!(
            harness.query_by_role_and_label(Role::Pane, pane).is_some(),
            "{pane}"
        );
    }
}
