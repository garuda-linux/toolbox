//! The picker and its buttons share one control height.

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui_kit::theme::PILL_HEIGHT;
use toolbox_services::diagnostics::page::Page;

use super::page;

#[test]
fn the_picker_and_collect_are_one_control_height_on_one_line() {
    let harness = page(Page::default());
    let picker = harness
        .get_by_role_and_label(Role::ComboBox, "Report to collect")
        .rect();

    let collect = harness
        .get_by_role_and_label(Role::Button, "Collect")
        .rect();

    for rect in [picker, collect] {
        assert!((rect.height() - PILL_HEIGHT).abs() < 0.5, "{rect:?}");
    }

    assert!(
        (picker.center().y - collect.center().y).abs() < 0.5,
        "{picker:?} {collect:?}"
    );
}
