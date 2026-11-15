use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;

use super::{Shown, page, system};

fn packs_tab() -> Shown {
    let mut harness = page(system());
    harness
        .get_by_role_and_label(Role::RadioButton, "Language packs")
        .click();
    harness.run();
    harness
}

fn search(harness: &mut Shown, text: &str) {
    harness.get_by_label("Search language packs").click();
    harness.run();
    harness
        .get_by_label("Search language packs")
        .type_text(text);
    harness.run();
}

#[test]
fn the_packs_search_finds_a_pack_by_its_program() {
    let mut harness = packs_tab();
    search(&mut harness, "firefox");

    assert!(
        harness
            .query_by_role_and_label(Role::CheckBox, "firefox-i18n-en-us")
            .is_some()
    );
}

#[test]
fn a_packs_search_without_a_match_offers_to_clear_it() {
    let mut harness = packs_tab();
    search(&mut harness, "klingon");

    assert!(harness.query_by_label("No language pack matches").is_some());
    harness
        .query_all_by_role_and_label(Role::Button, "Clear search")
        .last()
        .expect("the empty state's Clear search")
        .click();
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::CheckBox, "firefox-i18n-en-us")
            .is_some()
    );
}

/// The banner's count and the system language's choices come from the
/// cache while the system and the queue stay.
#[test]
fn a_frame_with_the_same_system_builds_nothing_again() {
    let mut harness = page(system());
    let system_built = harness.state().page.system.builds();
    harness.step();
    harness.step();
    assert_eq!(harness.state().page.system.builds(), system_built);

    let mut packs = packs_tab();
    let packs_built = packs.state().page.packs.builds();
    packs.step();
    packs.step();
    assert_eq!(packs.state().page.packs.builds(), packs_built);
}

/// `en_US.UTF-8` names its character set, `de_DE` does not.
#[test]
fn a_locale_names_its_character_set_once() {
    let mut harness = page(system());

    assert!(harness.query_by_label_contains(", UTF-8").is_none());
    assert!(
        harness
            .query_by_label("en_US.UTF-8, system language")
            .is_some()
    );

    harness.get_by_label("UTF-8 only").click();
    harness.run();
    assert!(harness.query_by_label("de_DE, ISO-8859-1").is_some());
}

#[test]
fn a_locale_search_without_a_match_offers_to_clear_it() {
    let mut harness = page(system());
    harness.get_by_label("Search locales").click();
    harness.run();
    harness.get_by_label("Search locales").type_text("klingon");
    harness.run();

    harness
        .query_all_by_role_and_label(Role::Button, "Clear search")
        .last()
        .expect("the empty state's Clear search")
        .click();
    harness.run();

    assert!(harness.query_by_label("No locale matches").is_none());
}
