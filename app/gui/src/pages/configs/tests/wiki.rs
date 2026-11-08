use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;

use super::{listed, page};
use crate::pages::Refusals;

#[test]
fn the_wiki_page_waits_in_the_menu_named_after_its_file() {
    let mut harness = page(listed(), Refusals::default());
    harness.run();
    harness.run();

    assert!(harness.query_by_label("Open wiki page").is_none());
    harness
        .get_by_role_and_label(Role::Button, "More actions for GRUB default")
        .click();
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::MenuItem, "Open wiki page")
            .is_some()
    );
}
