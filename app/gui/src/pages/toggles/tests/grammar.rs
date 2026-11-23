//! One row grammar by meaning: on a page that mixes them, a package
//! installs and removes with a button and a service enables with one. A
//! state that was not read counts as "?".

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::Toggle;
use toolbox_core::domains::Packages;
use toolbox_core::testing::Queue as _;
use toolbox_plan::PackageName;

use super::super::TogglePage;
use super::{Plain, page, shipped, unread};
use crate::catalogs::CatalogId;
use crate::command::{AppCommand, Command};
use crate::testing::{PageFixture, PageState};

fn harness_of(
    fixture: PageFixture,
    search: &str,
) -> egui_kittest::Harness<'static, PageState<Plain>> {
    let mut page = TogglePage::new(CatalogId::Components, "page-gaming", "page-gaming-subtitle");
    search.clone_into(&mut page.search);
    let mut harness = fixture.harness(Plain(page));
    harness.run();
    harness
}

#[test]
fn a_package_installs_with_a_button_and_a_service_enables_with_one() {
    let harness = page(CatalogId::Components, "");

    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Install PipeWire")
            .is_some()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::CheckBox, "PipeWire")
            .is_none()
    );

    let bluetooth = harness_of(shipped(), "bluetooth service");
    assert!(
        bluetooth
            .query_by_role_and_label(Role::Button, "Enable Bluetooth service")
            .is_some()
    );
}

#[test]
fn a_click_on_install_queues_the_package() {
    let mut harness = page(CatalogId::Components, "pipewire");

    harness
        .get_by_role_and_label(Role::Button, "Install PipeWire")
        .click();
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(AppCommand::ToggleEntry {
            catalog: CatalogId::Components,
            entry: "pipewire-support".to_owned(),
            wish: Toggle::On,
        })]
    );
}

/// Unlike a switch's row, a click on a button row's empty part is no
/// install the user meant.
#[test]
fn a_click_beside_the_button_queues_nothing() {
    let mut harness = page(CatalogId::Components, "pipewire");

    harness
        .get_by_role_and_label(Role::ListItem, "PipeWire")
        .click();
    harness.run();

    assert_eq!(harness.state().fixture.sent(), []);
}

#[test]
fn a_queued_package_offers_take_back_and_says_it_installs() {
    let mut fixture = shipped();
    let snapshot = std::sync::Arc::clone(&fixture.snapshot);
    fixture.changes.want::<Packages>(
        PackageName::new("pipewire-support").expect("a package name"),
        Toggle::On,
        &snapshot,
    );

    let harness = harness_of(fixture, "pipewire");

    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Take Back PipeWire")
            .is_some()
    );
    assert!(harness.query_by_label("Installs with Apply").is_some());
}

#[test]
fn an_unread_section_counts_a_question_mark_and_its_button_says_why() {
    let harness = harness_of(unread(), "");

    let audio = harness
        .query_all_by_role(Role::Button)
        .filter_map(|node| node.accesskit_node().label())
        .find(|label| label.starts_with("Audio "));

    assert_eq!(audio.as_deref(), Some("Audio ?/8"));

    assert!(
        harness
            .query_all_by_role(Role::Button)
            .filter_map(|node| node.accesskit_node().label())
            .any(|label| label.starts_with("All ?/"))
    );

    let bluetooth = harness_of(unread(), "bluetooth service");
    let button = bluetooth.get_by_role_and_label(Role::Button, "Enable Bluetooth service");
    assert_eq!(
        button.accesskit_node().description().as_deref(),
        Some("State not known.")
    );
}
