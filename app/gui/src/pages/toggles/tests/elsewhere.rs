//! A picked chip plus a search that only other sections match names the
//! section and offers to show every match, keeping the search.

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::page;
use crate::catalogs::CatalogId;

#[test]
fn a_picked_section_without_a_match_offers_all_matches() {
    let mut harness = page(CatalogId::Components, "pipewire");
    harness.state_mut().page.0.section = Some("printing".to_owned());
    harness.run();

    assert!(
        harness
            .query_by_label("Nothing in Printing matches")
            .is_some()
    );
    assert_eq!(
        harness
            .query_all_by_role_and_label(Role::Button, "Clear Search")
            .count(),
        1,
        "only the search field's own clear button shows"
    );

    let button = harness
        .query_all_by_role(Role::Button)
        .filter_map(|node| node.accesskit_node().label())
        .find(|label| label.starts_with("Show "));

    assert!(button.is_some(), "no Show all button");

    harness
        .get_by_role_and_label(Role::Button, &button.unwrap_or_default())
        .click();
    harness.run();

    assert_eq!(harness.state().page.0.section, None);
    assert_eq!(harness.state().page.0.search, "pipewire");
    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Install PipeWire")
            .is_some()
    );
}

#[test]
fn without_a_picked_section_the_empty_state_clears_the_search() {
    let mut harness = page(CatalogId::Components, "zz-nothing-matches");

    assert!(
        harness
            .query_by_label("Nothing matches “zz-nothing-matches”")
            .is_some()
    );
    harness
        .query_all_by_role_and_label(Role::Button, "Clear Search")
        .last()
        .expect("the empty state's Clear search")
        .click();
    harness.run();

    assert_eq!(harness.state().page.0.search, "");
}
