use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_plan::UnitScope;

use super::{
    Tabbed, TabbedIds, field_width, refresh_header_reading, search, tabbed_header_reading,
};

const IDS: TabbedIds<'static> = TabbedIds {
    title: "page-services",
    subtitle: "page-services-subtitle",
    tabs: "services-scope",
    refresh: "services-refresh",
};

fn tabbed() -> Harness<'static, Option<Tabbed<UnitScope>>> {
    let mut harness = toolbox_gui_kit::testing::harness(900.0, 200.0).build_ui_state(
        |ui, done: &mut Option<Tabbed<UnitScope>>| {
            let scopes = [UnitScope::System, UnitScope::User];
            *done = Some(tabbed_header_reading(
                ui,
                IDS,
                &scopes,
                UnitScope::System,
                false,
            ));
        },
        None,
    );

    harness.run();
    harness
}

#[test]
fn a_tab_is_named_by_its_text_id() {
    let harness = tabbed();

    assert!(harness.query_by_label("System").is_some());
    assert!(harness.query_by_label("User").is_some());
}

#[test]
fn a_click_on_another_tab_is_handed_back() {
    let mut harness = tabbed();

    harness.get_by_label("User").click();
    harness.step();

    assert_eq!(
        *harness.state(),
        Some(Tabbed {
            picked: Some(UnitScope::User),
            refresh: false,
        })
    );
}

#[test]
fn f5_reads_again() {
    let mut harness = toolbox_gui_kit::testing::harness(900.0, 200.0).build_ui_state(
        |ui, pressed: &mut bool| {
            *pressed |= refresh_header_reading(
                ui,
                ("page-kernels", "kernels-subtitle"),
                "kernels-refresh",
                false,
            );
        },
        false,
    );

    harness.run();
    assert!(!*harness.state());

    harness.key_press(egui::Key::F5);
    harness.run();
    assert!(*harness.state());
}

#[test]
fn the_search_is_named_and_clears_with_the_shared_button() {
    let mut harness = toolbox_gui_kit::testing::harness(900.0, 200.0).build_ui_state(
        |ui, text: &mut String| _ = search(ui, text, "services-search", "services-search-hint"),
        "ssh".to_owned(),
    );

    harness.run();
    harness
        .get_by_role_and_label(Role::Button, "Clear search")
        .click();
    harness.run();

    assert_eq!(harness.state(), "");
}

/// While a read runs Read again stays enabled, is named Reading, and a
/// click or F5 starts no second read.
#[test]
fn read_again_waits_while_a_read_runs() {
    let mut harness = toolbox_gui_kit::testing::harness(900.0, 200.0).build_ui_state(
        |ui, pressed: &mut bool| {
            *pressed |= refresh_header_reading(
                ui,
                ("page-kernels", "kernels-subtitle"),
                "kernels-refresh",
                true,
            );
        },
        false,
    );

    harness.run_steps(3);
    let button = harness.get_by_role_and_label(Role::Button, "Reading");
    assert!(!button.accesskit_node().is_disabled());

    button.click();
    harness.run_steps(3);
    assert!(!*harness.state());

    harness.key_press(egui::Key::F5);
    harness.run_steps(3);
    assert!(!*harness.state());
}

#[test]
fn a_search_beside_a_sort_keeps_the_width_the_row_leaves() {
    assert!((field_width(900.0, 240.0) - 240.0).abs() < f32::EPSILON);
    assert!((field_width(180.0, 240.0) - 180.0).abs() < f32::EPSILON);
    assert!((field_width(40.0, 240.0) - 120.0).abs() < f32::EPSILON);
}

#[test]
fn a_tabbed_read_again_waits_while_a_read_runs() {
    let mut harness = toolbox_gui_kit::testing::harness(900.0, 200.0).build_ui_state(
        |ui, done: &mut Option<Tabbed<UnitScope>>| {
            let scopes = [UnitScope::System, UnitScope::User];
            *done = Some(tabbed_header_reading(
                ui,
                IDS,
                &scopes,
                UnitScope::System,
                true,
            ));
        },
        None,
    );

    harness.run_steps(3);
    harness.key_press(egui::Key::F5);
    harness.run_steps(3);

    assert!(harness.state().is_some_and(|done| !done.refresh));
}
