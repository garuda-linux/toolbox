//! The list beside the details, the details alone on a narrow page, and the
//! keys that move the selection.

use egui::accesskit::Role;
use egui::{Key, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::PackagesPage;
use super::{fixture, gimp_details, name, open_row};
use crate::testing::PageState;

/// The list on the start view, with gimp's details offered on its row.
pub(super) fn list_with_gimp() -> Harness<'static, PageState<PackagesPage>> {
    let mut harness = fixture(Some(gimp_details())).harness(PackagesPage::default());
    harness.run();
    harness
}

#[test]
fn a_wide_page_keeps_the_list_beside_the_details_of_the_open_row() {
    let mut harness = list_with_gimp();
    open_row(&mut harness, "gimp");

    let list = harness
        .get_by_role_and_label(Role::ListItem, "leftover")
        .rect();

    let title = harness.get_by_role_and_label(Role::Heading, "gimp").rect();
    assert!(list.right() < title.left(), "{list:?} {title:?}");
    assert!(harness.query_by_label("Back to Packages").is_none());
}

#[test]
fn a_narrow_page_shows_the_details_alone_and_escape_returns_to_the_list() {
    let mut harness = list_with_gimp();
    open_row(&mut harness, "gimp");

    harness.set_size(vec2(480.0, 480.0));
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(Role::ListItem, "leftover")
            .is_none()
    );
    assert!(harness.query_by_label("Back to Packages").is_some());

    harness.key_press(Key::Escape);
    harness.run();
    harness.run();
    assert_eq!(harness.state().page.open, None);
    assert!(
        harness
            .query_by_role_and_label(Role::ListItem, "leftover")
            .is_some()
    );
}

#[test]
fn the_arrow_keys_move_the_selection_and_the_details_follow() {
    let mut harness = list_with_gimp();
    open_row(&mut harness, "gimp");

    harness.key_press(Key::ArrowDown);
    harness.run();

    // The start view lists gegl, gimp, leftover and paru, so Down from gimp
    // lands on leftover, whose details replace gimp's.
    assert_eq!(harness.state().page.open, Some(name("leftover")));
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "leftover")
            .is_some()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "gimp")
            .is_none()
    );
    assert_eq!(
        harness.state().page.selection(false).as_deref(),
        Some("leftover")
    );
    assert!(
        harness
            .get_by_role_and_label(Role::ListItem, "leftover")
            .is_focused()
    );
}
