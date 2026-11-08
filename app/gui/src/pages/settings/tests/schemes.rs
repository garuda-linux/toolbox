use std::path::Path;

use egui::Key;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_catalog::{ColourSchemeCatalog, SchemeMode};
use toolbox_config::Settings;
use toolbox_plan::SchemeId;

use super::super::SettingsPage;
use crate::catalogs::Catalogs;
use crate::command::Command;
use crate::settings::SettingsCommand;
use crate::testing::{PageFixture, PageState};

const DARK: &str = "Dark colour scheme";
const LIGHT: &str = "Light colour scheme";

fn page(settings: Settings) -> Harness<'static, PageState<SettingsPage>> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog");
    let mut harness = PageFixture {
        settings,
        catalogs: Catalogs {
            colour_schemes: ColourSchemeCatalog::load(&dir).unwrap(),
            ..Catalogs::default()
        },
        ..PageFixture::default()
    }
    .harness(SettingsPage::default());

    harness.run();
    harness
}

fn sent(harness: &Harness<'static, PageState<SettingsPage>>) -> Vec<Command> {
    harness.state().fixture.sent()
}

fn scheme(mode: SchemeMode, id: Option<&str>) -> Command {
    let id = id.map(|id| SchemeId::new(id).unwrap());
    Command::from(SettingsCommand::Scheme(mode, id))
}

fn open(harness: &mut Harness<'static, PageState<SettingsPage>>, picker: &str) {
    harness
        .get_by_role_and_label(Role::ComboBox, picker)
        .click();
    harness.run();
}

fn shown(harness: &Harness<'static, PageState<SettingsPage>>, picker: &str) -> String {
    harness
        .get_by_role_and_label(Role::ComboBox, picker)
        .accesskit_node()
        .value()
        .unwrap_or_default()
}

#[test]
fn both_pickers_sit_under_theme_and_start_on_system() {
    let harness = page(Settings::default());

    let theme = harness.get_by_role_and_label(Role::Label, "Theme").rect();
    let dark = harness.get_by_role_and_label(Role::ComboBox, DARK).rect();
    let light = harness.get_by_role_and_label(Role::ComboBox, LIGHT).rect();

    assert!(theme.bottom() <= dark.top(), "{theme:?} then {dark:?}");
    assert!(dark.bottom() <= light.top(), "{dark:?} then {light:?}");
    assert_eq!(shown(&harness, DARK), "System");
    assert_eq!(shown(&harness, LIGHT), "System");
}

#[test]
fn each_picker_lists_system_first_then_only_its_modes_schemes() {
    let mut harness = page(Settings::default());

    open(&mut harness, DARK);
    for name in [
        "Catppuccin Mocha",
        "Nord",
        "Dracula",
        "Sweet",
        "Tokyo Night",
    ] {
        assert!(harness.query_by_label(name).is_some(), "{name}");
    }

    assert!(harness.query_by_label("Catppuccin Latte").is_none());

    harness.key_press(Key::Escape);
    harness.run();
    open(&mut harness, LIGHT);
    for name in [
        "Catppuccin Latte",
        "Tokyo Night Day",
        "Nord light",
        "Dracula Alucard",
    ] {
        assert!(harness.query_by_label(name).is_some(), "{name}");
    }

    assert!(harness.query_by_label("Dracula").is_none());
}

#[test]
fn a_picked_scheme_is_sent_for_its_mode() {
    let mut harness = page(Settings::default());

    open(&mut harness, DARK);
    harness.get_by_label("Nord").click();
    harness.run();
    open(&mut harness, LIGHT);
    harness.get_by_label("Catppuccin Latte").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [
            scheme(SchemeMode::Dark, Some("nord")),
            scheme(SchemeMode::Light, Some("catppuccin-latte")),
        ]
    );
}

#[test]
fn system_after_a_picked_scheme_takes_the_desktops_colours_back() {
    let mut settings = Settings::default();
    settings.colours.dark_scheme = Some(SchemeId::new("dracula").unwrap());
    let mut harness = page(settings);
    assert_eq!(shown(&harness, DARK), "Dracula");

    open(&mut harness, DARK);
    harness
        .get_by_role_and_label(Role::ListBoxOption, "System")
        .click();
    harness.run();

    assert_eq!(sent(&harness), [scheme(SchemeMode::Dark, None)]);
}

#[test]
fn a_scheme_the_catalog_lost_shows_as_system() {
    let mut settings = Settings::default();
    settings.colours.light_scheme = Some(SchemeId::new("removed-scheme").unwrap());

    assert_eq!(shown(&page(settings), LIGHT), "System");
}

#[test]
fn a_scheme_is_picked_with_the_keyboard_alone() {
    let mut harness = page(Settings::default());

    harness.get_by_role_and_label(Role::ComboBox, DARK).focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();

    for _ in 0_u8..2 {
        harness.key_press(Key::Tab);
        harness.run();
    }

    harness.key_press(Key::Enter);
    harness.run();

    assert_eq!(
        sent(&harness),
        [scheme(SchemeMode::Dark, Some("catppuccin-frappe"))]
    );
}

#[test]
fn the_german_pickers_fit_their_rows() {
    let mut harness = page(Settings::default());
    crate::i18n::set_language(&harness.ctx, &"de-DE".parse().unwrap());
    harness.run();

    for (picker, title) in [
        ("Dunkle Farben", "Dunkle Farben"),
        ("Helle Farben", "Helle Farben"),
    ] {
        let pill = harness.get_by_role_and_label(Role::ComboBox, picker).rect();
        let label = harness.get_by_role_and_label(Role::Label, title).rect();
        assert!(label.right() < pill.left(), "{label:?} beside {pill:?}");
    }
}
