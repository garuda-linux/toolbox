use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::{kitty, open_tab, page, programs, with};
use crate::maintenance::Maintenance;

#[test]
fn a_reset_row_names_the_files_it_replaces_in_sight() {
    let mut harness = page(with(
        programs(),
        Maintenance::showing(None, false, vec![kitty()], None),
    ));

    open_tab(&mut harness, "Reset settings");

    let row = harness.get_by_role_and_label(Role::CheckBox, "Kitty");
    assert_eq!(
        row.accesskit_node().description().as_deref(),
        Some("~/.config/kitty/kitty.conf")
    );
    assert!(row.rect().height() > 40.0, "the file line has no room");
}
