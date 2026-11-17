use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::{REINSTALL, WINDOW, show};

// Not `INSTALLED_DIR`, which a Nix build points into the store.
const FOLDER: &str = "/usr/share/garuda-toolbox/catalog";

fn harness() -> Harness<'static> {
    Harness::builder()
        .with_size(egui::Vec2::from(WINDOW))
        .build_ui(|ui| {
            toolbox_gui_kit::install(ui.ctx());
            egui::CentralPanel::default().show(ui, |ui| {
                show(ui, std::path::Path::new(FOLDER));
            });
        })
}

#[test]
fn the_state_names_the_missing_folder_and_offers_quit() {
    let mut harness = harness();
    harness.run();

    assert!(harness.query_by_label_contains(FOLDER).is_some());
    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Quit")
            .is_some()
    );

    harness.snapshot("missing/catalog_dark");
}

#[test]
fn the_folder_and_the_reinstall_command_can_be_copied() {
    let mut harness = harness();
    harness.run();

    assert!(harness.query_by_label_contains(REINSTALL).is_some());
    for name in ["Copy Missing folder", "Copy Reinstall with"] {
        assert!(
            harness
                .query_by_role_and_label(Role::Button, name)
                .is_some(),
            "no {name} button"
        );
    }
}

#[test]
fn the_state_stands_in_the_middle_of_the_window() {
    let mut harness = harness();
    harness.run();

    let title = harness
        .get_by_label("Garuda Toolbox is not installed completely")
        .rect();

    let quit = harness.get_by_role_and_label(Role::Button, "Quit").rect();
    let [width, height] = WINDOW;

    assert!(
        (title.center().x - width / 2.0).abs() < 1.0,
        "the title {title:?} is not centred in {width}"
    );
    assert!(
        (quit.center().x - width / 2.0).abs() < 1.0,
        "Quit {quit:?} is not centred in {width}"
    );

    // The icon stands above the title, so the title starts further from the
    // top than Quit ends from the bottom.
    let (above, below) = (title.top(), height - quit.bottom());
    assert!(
        below > 16.0 && above > below && above - below < 120.0,
        "the state runs from {above} to {below} above the bottom"
    );
}
