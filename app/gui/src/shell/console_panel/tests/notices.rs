use egui_kittest::kittest::Queryable as _;

use crate::console::Console;

const HINT: &str = "Save and quit the editor to go on.";

fn notices(editing: bool) -> egui_kittest::Harness<'static, Console> {
    let mut harness = toolbox_gui_kit::testing::harness(600.0, 200.0).build_ui_state(
        move |ui, console: &mut Console| super::super::notices(ui, console, editing),
        Console::new(),
    );

    harness.run();
    harness
}

/// The page under the sheet says how an edit goes on, but the sheet covers
/// it while the editor runs.
#[test]
fn a_running_config_editor_tells_how_to_go_on() {
    assert!(notices(true).query_by_label_contains(HINT).is_some());
}

#[test]
fn other_runs_show_no_editor_hint() {
    assert!(notices(false).query_by_label_contains(HINT).is_none());
}
