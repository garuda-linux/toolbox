use std::sync::Arc;

use egui::Theme;
use egui::accesskit::{Role, Toggled};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use parking_lot::Mutex;
use toolbox_config::{RefreshInterval, Settings};
use toolbox_gui::settings::SettingsStore;
use toolbox_gui::{App, PreferencesSetup, Setup};

use crate::common::{TALL, window};

#[derive(Debug, Default)]
pub struct Kept {
    saved: Vec<Settings>,
    autostart: Vec<bool>,
}

#[derive(Debug, Default)]
pub struct Recorder(Mutex<Kept>);

impl Recorder {
    /// Every settings file the app asked to write, the newest last.
    pub fn saved(&self) -> Vec<Settings> {
        self.0.lock().saved.clone()
    }
}

impl SettingsStore for Recorder {
    fn save(&self, settings: Settings) {
        self.0.lock().saved.push(settings);
    }

    fn set_autostart(&self, on: bool) {
        self.0.lock().autostart.push(on);
    }
}

pub fn settings() -> (Harness<'static, App>, Arc<Recorder>) {
    let store = Arc::new(Recorder::default());
    let kept = Arc::clone(&store);
    let mut harness = window(TALL).build_eframe(move |cc| {
        let setup = Setup {
            preferences: PreferencesSetup {
                store: Some(kept),
                ..PreferencesSetup::default()
            },
            ..Setup::default()
        };

        App::new(cc, setup)
    });

    harness.run();
    harness.get_by_label("Settings").click();
    harness.run();
    (harness, store)
}

/// Clicks `option` of the segmented control `group`, scrolled into view
/// first, as the lower rows lie below the window at first.
pub fn segment(harness: &mut Harness<'static, App>, group: &str, option: &str) {
    harness
        .get_by_role_and_label(Role::RadioGroup, group)
        .get_by_role_and_label(Role::RadioButton, option)
        .scroll_to_me();
    harness.run();
    harness
        .get_by_role_and_label(Role::RadioGroup, group)
        .get_by_role_and_label(Role::RadioButton, option)
        .click();
    harness.run();
}

#[test]
fn a_theme_picked_shows_at_once() {
    let (mut harness, _store) = settings();

    segment(&mut harness, "Theme", "Light");

    assert_eq!(harness.ctx.theme(), Theme::Light);
}

#[test]
fn reduced_motion_picked_stops_the_motion() {
    let (mut harness, _store) = settings();

    segment(&mut harness, "Reduced motion", "On");

    assert!(toolbox_gui_kit::theme::reduced_motion(&harness.ctx));
}

#[test]
fn start_at_login_reaches_the_store() {
    let (mut harness, store) = settings();

    harness
        .get_by_role_and_label(Role::CheckBox, "Start at login")
        .click();
    harness.run();

    assert_eq!(store.0.lock().autostart, [true]);
}

#[test]
fn only_when_i_ask_is_saved() {
    let (mut harness, store) = settings();

    harness
        .get_by_role_and_label(Role::CheckBox, "Only when I ask")
        .click();
    harness.run();

    let saved = &store.0.lock().saved;
    assert_eq!(
        saved.last().map(|settings| settings.status.refresh),
        Some(RefreshInterval::Off)
    );
}

#[test]
fn the_services_page_opens_on_the_users_units_at_once() {
    let (mut harness, _store) = settings();

    segment(&mut harness, "Services page opens on", "My account");
    harness.get_by_label("Services").click();
    harness.run();

    let user = harness
        .get_by_role_and_label(Role::RadioGroup, "Services to show")
        .get_by_role_and_label(Role::RadioButton, "User");

    assert_eq!(user.accesskit_node().toggled(), Some(Toggled::True));
}

#[test]
fn another_language_changes_the_texts_at_once() {
    let (mut harness, _store) = settings();

    harness
        .get_by_role_and_label(Role::ComboBox, "Language")
        .click();
    harness.run();
    harness.get_by_label("Deutsch (Deutschland)").click();
    harness.run();

    assert!(harness.query_all_by_label("Einstellungen").next().is_some());
}

#[test]
fn a_setting_select_takes_the_narrow_control_column() {
    let (harness, _store) = settings();
    let language = harness
        .get_by_role_and_label(Role::ComboBox, "Language")
        .rect()
        .width();

    let narrow = toolbox_gui_kit::tokens::Sizes::DEFAULT.setting_control_narrow;
    assert!(
        (language - narrow).abs() < 0.5,
        "{language} against {narrow}"
    );
}
