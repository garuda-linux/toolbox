use egui::Key;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::features::boot::edit::Edit;

use super::{edit, ready, sent};

/// Enter adds the parameter and leaves the field ready for the next one.
#[test]
fn enter_adds_the_parameter_and_keeps_the_field_focused() {
    let mut harness = ready();
    harness.get_by_label("New kernel parameter").scroll_to_me();
    harness.run();
    harness.get_by_label("New kernel parameter").click();
    harness.run();
    harness
        .get_by_label("New kernel parameter")
        .type_text("nowatchdog");
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();

    assert_eq!(
        sent(&harness),
        [edit(Edit::AddParameter {
            text: "nowatchdog".to_owned(),
            confirmed: false,
        })]
    );
    assert!(harness.get_by_label("New kernel parameter").is_focused());
}
