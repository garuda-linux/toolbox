use egui_kittest::kittest::Queryable as _;

use super::UiExt as _;
use crate::icons::GO_HOME;
use crate::testing::harness;

#[test]
fn an_icon_button_is_found_and_pressed_by_its_alt_text() {
    let mut harness = harness(200.0, 100.0).build_ui_state(
        |ui, clicked: &mut bool| {
            if crate::widgets::icon_button(ui, GO_HOME, "Home").clicked() {
                *clicked = true;
            }
        },
        false,
    );

    harness.get_by_label("Home").click();
    harness.run();

    assert!(*harness.state());
}

#[test]
fn an_icon_has_its_alt_text_as_name() {
    let mut harness = harness(200.0, 100.0).build_ui(|ui| {
        ui.icon(GO_HOME, "Home page");
    });

    harness.run();

    assert!(harness.query_by_label("Home page").is_some());
}

#[test]
fn a_warning_label_is_named_by_its_text() {
    let mut harness = harness(300.0, 200.0).build_ui(|ui| {
        ui.warning_label("Careful");
    });

    harness.run();

    assert!(harness.query_by_label("Careful").is_some());
}

#[test]
fn a_loading_indicator_is_named_by_its_text() {
    let mut harness = harness(300.0, 100.0).build_ui(|ui| {
        ui.loading("Reading packages");
    });

    harness.run_ok();

    assert!(harness.query_by_label("Reading packages").is_some());
}

#[test]
fn a_named_pane_is_found_by_its_name() {
    let mut harness = harness(300.0, 200.0).build_ui(|ui| {
        egui::Panel::left("pages").show(ui, |ui| {
            ui.name_pane("Pages");
            ui.label("Home");
        });
    });

    harness.run();

    assert!(
        harness
            .query_by_role_and_label(egui::accesskit::Role::Pane, "Pages")
            .is_some()
    );
}
