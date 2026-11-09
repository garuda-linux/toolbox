use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::show;

fn harness() -> Harness<'static> {
    Harness::builder()
        .with_size(egui::vec2(800.0, 300.0))
        .build_ui(|ui| {
            toolbox_gui_kit::install(ui.ctx());
            egui::CentralPanel::default().show(ui, |ui| {
                show(ui, std::path::Path::new(toolbox_catalog::INSTALLED_DIR));
            });
        })
}

#[test]
fn the_banner_names_the_missing_folder_and_offers_quit() {
    let mut harness = harness();
    harness.run();

    assert!(
        harness
            .query_by_label_contains("/usr/share/garuda-toolbox/catalog")
            .is_some()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Quit")
            .is_some()
    );

    harness.snapshot("missing/catalog_dark");
}
