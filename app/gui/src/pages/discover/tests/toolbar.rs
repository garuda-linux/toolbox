//! The toolbar waits for the apps, and on a narrow row the search leaves
//! the sort its room.

use egui::accesskit::Role;
use egui::vec2;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::super::DiscoverPage;
use super::offered;
use crate::testing::{PageFixture, rect_of};

#[test]
fn search_and_sort_show_off_until_the_apps_are_listed() {
    let mut harness = PageFixture::default().harness(DiscoverPage::default());
    harness.run();

    let search = harness.get_by_label("Search apps");
    assert!(search.accesskit_node().is_disabled());
    let sort = harness.get_by_role_and_label(Role::ComboBox, "Sort apps");
    assert!(sort.accesskit_node().is_disabled());
}

#[test]
fn on_a_narrow_row_the_search_stops_before_the_sort() {
    let mut harness = offered().harness(DiscoverPage::default());
    harness.set_size(vec2(440.0, 480.0));
    harness.run();

    let search = rect_of(&harness, Role::TextInput, "Search apps");
    let sort = rect_of(&harness, Role::ComboBox, "Sort apps");
    assert!(search.right() < sort.left(), "{search:?} {sort:?}");
}
