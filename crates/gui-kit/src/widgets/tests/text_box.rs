use egui::TextEdit;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::text_box;
use crate::testing::harness;

const NAME: &str = "The report";

fn shown() -> Harness<'static> {
    let mut harness = harness(400.0, 300.0).build_ui(|ui| {
        let mut text = "first line\nsecond line\nthird line";
        text_box(ui, TextEdit::multiline(&mut text), NAME, 160.0);
    });

    harness.run();

    harness
}

#[test]
fn a_text_box_is_named_for_screen_readers() {
    let harness = shown();

    assert!(
        harness
            .query_by_role_and_label(Role::MultilineTextInput, NAME)
            .is_some()
    );
}
