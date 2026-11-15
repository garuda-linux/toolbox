use egui::Key;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;

use super::{focused, press, shown};
use crate::testing::tab_into;

#[test]
fn a_list_takes_one_tab_stop_and_home_and_end_reach_its_ends() {
    let mut harness = shown(&["a", "b", "c"], &[], 400.0);

    assert!(tab_into(&mut harness, Role::ListItem));
    press(&mut harness, Key::End);
    assert!(focused(&harness, "c"));

    press(&mut harness, Key::Home);
    assert!(focused(&harness, "a"));
}

#[test]
fn tab_from_the_stop_row_leaves_the_list_for_its_controls() {
    let mut harness = shown(&["a", "b"], &[], 400.0);
    assert!(tab_into(&mut harness, Role::ListItem));

    press(&mut harness, Key::Tab);

    assert!(
        harness
            .get_by_role_and_label(Role::CheckBox, "Switch a")
            .is_focused()
    );
}
