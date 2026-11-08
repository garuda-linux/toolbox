use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_plan::UnitScope;

use super::{Tabbed, TabbedIds, refresh_header, search, tabbed_header};

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
            *done = Some(tabbed_header(ui, IDS, &scopes, UnitScope::System));
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
            *pressed |= refresh_header(ui, "page-kernels", "kernels-subtitle", "kernels-refresh");
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
        .get_by_role_and_label(Role::Button, "Clear the search")
        .click();
    harness.run();

    assert_eq!(harness.state(), "");
}
