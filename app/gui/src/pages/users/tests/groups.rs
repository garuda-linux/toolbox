use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;

use super::{page, unlocked};

fn groups_tab() -> super::Shown {
    let mut harness = page(unlocked());
    harness
        .get_by_role_and_label(Role::RadioButton, "Groups")
        .click();
    harness.run();
    harness
}

/// `groupdel` refuses a group that is an account's primary one.
#[test]
fn a_primary_group_offers_no_removal() {
    let harness = groups_tab();

    assert!(harness.query_by_label("Remove ada…").is_none());
    assert!(harness.query_by_label("Remove video…").is_some());
}

/// Every Remove button reads the same, so the column stays straight. The
/// group goes only into the name a screen reader hears.
#[test]
fn the_remove_button_shows_one_word_and_names_its_group() {
    let harness = groups_tab();
    let button = harness.get_by_role_and_label(Role::Button, "Remove video…");

    // "Remove video…" in the button's font is wider than this.
    assert!(button.rect().width() < 110.0, "{:?}", button.rect());
}

#[test]
fn before_the_first_read_placeholder_rows_wait_on_both_tabs() {
    for (tab, name) in [
        ("Users", "Reading the accounts…"),
        ("Groups", "Reading the groups…"),
    ] {
        let mut harness = crate::testing::PageFixture {
            probed: false,
            ..crate::testing::PageFixture::default()
        }
        .harness(super::super::UsersPage::default());

        harness.run();
        harness
            .get_by_role_and_label(Role::RadioButton, tab)
            .click();
        harness.run();

        assert!(
            harness
                .query_by_role_and_label(Role::ProgressIndicator, name)
                .is_some(),
            "{tab}"
        );
    }
}
