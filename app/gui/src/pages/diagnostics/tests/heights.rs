use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_services::diagnostics::page::Page;

use super::{page, uploaded};

/// Darkly's push button at the test font, its text plus 18 px.
const PUSH_HEIGHT: f32 = 34.0;

#[test]
fn every_collect_is_one_push_button_height() {
    let harness = page(Page::default());
    for name in ["Collect System information", "Collect Full report"] {
        let collect = harness.get_by_role_and_label(Role::Button, name).rect();
        assert!(
            (collect.height() - PUSH_HEIGHT).abs() < 0.5,
            "{name}: {collect:?}"
        );
    }
}

#[test]
fn the_uploaded_text_centres_on_its_buttons() {
    let harness = page(uploaded());
    let text = harness.get_by_label("Uploaded.").rect();
    let copy = harness
        .get_by_role_and_label(Role::Button, "Copy Link")
        .rect();

    assert!(
        (text.center().y - copy.center().y).abs() < 0.75_f32,
        "{text:?} beside {copy:?}"
    );
}
