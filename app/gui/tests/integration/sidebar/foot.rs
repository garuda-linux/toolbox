use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui::page::PageId;

use super::{app, entry_name};
use crate::common::HEIGHT;

/// The app's own pages at the foot keep the list's right edge, which the
/// list's scroll strip sets, so their highlights do not overhang it.
#[test]
fn the_foot_entries_end_where_the_list_entries_do() {
    let mut harness = app(HEIGHT, false);
    harness.run();

    let home = entry_name(&harness, PageId::Home);
    let settings = entry_name(&harness, PageId::Settings);
    let list = harness.get_by_role_and_label(Role::Tab, &home).rect();
    let foot = harness.get_by_role_and_label(Role::Tab, &settings).rect();

    assert!(
        (foot.right() - list.right()).abs() < 0.5,
        "the foot ends at {} and the list at {}",
        foot.right(),
        list.right()
    );
}
