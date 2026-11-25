use super::{count_at, rules};

const RULE: &str = "scroll-bars";
const PRODUCTION: &str = "app/gui/src/shell/changes.rs";

#[test]
fn a_bare_show_of_a_scroll_area_is_reported() {
    for call in [
        "ScrollArea::vertical().show(ui, |ui| {});",
        "egui::ScrollArea::both().max_height(4.0).show(ui, |ui| {});",
        "ScrollArea::horizontal().show_viewport(ui, |ui, _| {});",
        "ScrollArea::vertical().show_rows(ui, 14.0, 80, |ui, _| {});",
    ] {
        let text = format!("fn f(ui: &mut Ui) {{\n{call}\n}}\n");

        assert_eq!(count_at(RULE, PRODUCTION, &text), 1, "{call}");
    }
}

#[test]
fn the_helpers_pass() {
    for call in [
        "show_scroll(ui, ScrollArea::vertical(), |ui| {});",
        "show_scroll_hidden(ui, ScrollArea::vertical(), |ui| {});",
        "show_scroll_rows(ui, ScrollArea::vertical(), 14.0, 80, |ui, _| {});",
    ] {
        let text = format!("fn f(ui: &mut Ui) {{\n{call}\n}}\n");

        assert_eq!(count_at(RULE, PRODUCTION, &text), 0, "{call}");
    }
}

#[test]
fn the_helper_module_and_test_files_pass() {
    let text = "fn f(ui: &mut Ui) {\nScrollArea::vertical().show(ui, |ui| {});\n}\n";

    for path in [
        "crates/gui-kit/src/theme/scrollbar.rs",
        "crates/gui-kit/src/theme/scrollbar/tests.rs",
        "app/gui/tests/integration/page_scroll.rs",
    ] {
        assert_eq!(count_at(RULE, path, text), 0, "{path}");
    }
}

#[test]
fn other_shows_pass_where_no_scroll_area_is_named() {
    let text = "fn f(ui: &mut Ui) {\nFrame::new().show(ui, |ui| {});\n}\n";

    assert_eq!(count_at(RULE, PRODUCTION, text), 0);
}

#[test]
fn a_frame_or_a_popup_beside_a_scroll_area_passes() {
    let text = "fn f(ui: &mut Ui) {\nlet a = ScrollArea::vertical();\nFrame::new().show(ui, |ui| {});\nPopup::new().show(ui, |ui| {});\n}\n";

    assert_eq!(count_at(RULE, PRODUCTION, text), 0);
}

#[test]
fn an_allow_directive_covers_an_egui_show_beside_a_scroll_area() {
    let text = "fn f(ui: &mut Ui) {\n\
        let area = ScrollArea::vertical();\n\
        // toolbox-lint: allow(scroll-bars) a popup, not a scroll area\n\
        Popup::new().show(ui, |ui| {});\n\
        }\n";

    assert_eq!(rules(text), Vec::<&str>::new());
}
