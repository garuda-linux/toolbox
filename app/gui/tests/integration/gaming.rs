use egui::accesskit::Role;
use egui::{Key, Modifiers, Theme};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use tokio::runtime::Runtime;
use toolbox_gui::App;

use crate::common::{queued_shown, type_into};
use crate::components::{components_on, with_pipewire};

pub fn gaming() -> (Harness<'static, App>, Runtime) {
    gaming_on(Theme::Dark)
}

/// The Gaming page drawn on `theme`, with that theme's desktop look.
pub fn gaming_on(theme: Theme) -> (Harness<'static, App>, Runtime) {
    let (mut harness, runtime) = components_on(with_pipewire(), theme);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Num4);
    harness.run();
    (harness, runtime)
}

#[test]
fn the_launchers_show_and_a_click_queues_one() {
    let (mut harness, _runtime) = gaming();
    assert!(
        harness
            .query_by_role_and_label(Role::List, "Game Launchers")
            .is_some()
    );

    // The list builds only the rows near the view, so the search brings
    // Steam up instead of a scroll.
    type_into(&mut harness, "Search this page", "Steam");
    harness
        .get_by_role_and_label(Role::Button, "Install Steam")
        .click();
    harness.run();

    assert!(queued_shown(&harness));
    assert!(harness.query_by_label("Installs with Apply").is_some());
}

#[test]
fn a_chip_keeps_one_section() {
    let (mut harness, _runtime) = gaming();

    crate::discover::chip(&mut harness, "WINE ");

    assert!(harness.query_by_label("Game Launchers").is_none());
    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Install Proton CachyOS")
            .is_some()
    );
}
