use std::collections::BTreeSet;

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::dialogs::type_into;
use super::{from_menu, group_name, name, sent};
use crate::command::{Command, UserEdit};

const WHEEL: &str = "wheel (group ID 998, members: 1)";

fn membership_dialog() -> super::Shown {
    from_menu("ada", "Groups…")
}

fn disabled(harness: &super::Shown, label: &str) -> bool {
    harness
        .get_by_role_and_label(Role::Button, label)
        .accesskit_node()
        .is_disabled()
}

#[test]
fn queue_changes_waits_until_the_groups_differ_from_the_start() {
    let mut harness = membership_dialog();
    assert!(disabled(&harness, "Queue Changes"));

    harness.get_by_role_and_label(Role::CheckBox, WHEEL).click();
    harness.run();
    assert!(!disabled(&harness, "Queue Changes"));

    harness.get_by_role_and_label(Role::CheckBox, WHEEL).click();
    harness.run();
    assert!(disabled(&harness, "Queue Changes"));
}

#[test]
fn the_group_search_narrows_the_boxes_and_keeps_the_ticks() {
    let mut harness = membership_dialog();
    type_into(&mut harness, Role::TextInput, "Search groups", "whe");

    assert!(
        harness
            .query_by_role_and_label(Role::CheckBox, "video (group ID 1500, members: 1)")
            .is_none()
    );

    harness.get_by_role_and_label(Role::CheckBox, WHEEL).click();
    harness.run();
    harness.get_by_label("Queue Changes").click();
    harness.run();

    let groups = ["ada", "video", "wheel"].map(group_name);
    assert_eq!(
        sent(&harness),
        [Command::from(UserEdit::SetGroups {
            user: name("ada"),
            groups: BTreeSet::from(groups),
        })]
    );
}
