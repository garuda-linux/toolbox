//! The pages with the longest German texts, where labels clip first.

use egui::Theme;
use egui::accesskit::Role;
use egui_kittest::SnapshotResults;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui::Setup;
use toolbox_gui::page::PageId;

use super::{WIDE, page};

fn german() -> Setup {
    Setup {
        system_language: "de-DE".parse().unwrap(),
        ..Setup::default()
    }
}

#[test]
fn the_longest_pages_in_german() {
    let mut results = SnapshotResults::new();
    let plain = [
        PageId::Services,
        PageId::Components,
        PageId::SystemSettings,
        PageId::Diagnostics,
    ];

    for id in plain {
        let mut harness = page(id, Theme::Dark, WIDE, german());
        results.add(harness.try_snapshot(format!("coverage/{}_german", id.slug())));
    }

    let mut maintenance = page(PageId::Maintenance, Theme::Dark, WIDE, german());
    for (tab, name) in [
        ("Allgemein", "common"),
        ("Einstellungen zurücksetzen", "reset"),
        ("Reparaturen", "repairs"),
    ] {
        maintenance
            .get_by_role_and_label(Role::RadioButton, tab)
            .click();
        maintenance.run();
        results.add(maintenance.try_snapshot(format!("coverage/maintenance_{name}_german")));
    }

    let mut setup = page(PageId::Setup, Theme::Dark, WIDE, german());
    setup.get_by_label("Weiter").click();
    setup.run();
    results.add(setup.try_snapshot("coverage/setup_step_german"));

    // The Users page with its accounts read and the add dialog open: its
    // labels are the longest of the window.
    let mut users = crate::users::users();
    let harness = users.harness();
    in_german(harness);
    harness.get_by_label("Benutzer hinzufügen…").click();
    harness.run();
    results.add(harness.try_snapshot("coverage/users_add_german"));

    let mut boot = crate::boot::app();
    crate::boot::open_boot(boot.harness());
    in_german(boot.harness());
    results.add(boot.harness().try_snapshot("coverage/boot_menu_german"));
}

/// Switches a fixture read in English to German, as a language change in
/// Settings does.
fn in_german(harness: &mut egui_kittest::Harness<'static, toolbox_gui::App>) {
    let de = "de-DE".parse().unwrap();
    toolbox_gui::i18n::set_language(&harness.ctx, &de);
    harness.run();
}
