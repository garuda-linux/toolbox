//! The toolbar row under the page header looks the same on every page.

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_gui::Setup;
use toolbox_gui_kit::testing::painted_text;

use crate::common::launch;

/// The sort dropdown `sort` on the page behind the sidebar entry `page`,
/// shown as a whole value and not cut.
fn sort_value_is_whole(page: &str, sort: &str) {
    let mut harness = launch(Setup::default());

    harness.get_by_role_and_label(Role::Tab, page).click();
    harness.run();

    let value = harness
        .get_by_role_and_label(Role::ComboBox, sort)
        .accesskit_node()
        .value()
        .expect("the dropdown names its value");

    let text = painted_text(&harness, &value).expect("the value is painted");

    assert!(!text.galley.elided, "{sort} cuts {value}");
}

#[test]
fn the_app_catalog_sort_shows_its_value_whole() {
    sort_value_is_whole("App Catalog", "Sort apps");
}
