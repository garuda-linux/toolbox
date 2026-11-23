use std::sync::Arc;

use egui::Theme;
use egui::accesskit::{Role, Toggled};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use parking_lot::Mutex;
use toolbox_config::{RefreshInterval, Settings};
use toolbox_gui::settings::SettingsStore;
use toolbox_gui::{App, PreferencesSetup, Setup};

use crate::common::{TALL, shared_runtime, window};
use crate::look::look_for;

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
    settings_on(Theme::Dark)
}

/// The Settings page drawn on `theme`, with that theme's desktop look.
pub fn settings_on(theme: Theme) -> (Harness<'static, App>, Arc<Recorder>) {
    settings_on_with(theme, false)
}

/// The Settings page where the compositor `desktop_blur`s behind the window.
/// The window starts translucent as `main` makes it, when both the setting
/// and the compositor allow it.
pub fn settings_on_with(
    theme: Theme,
    desktop_blur: bool,
) -> (Harness<'static, App>, Arc<Recorder>) {
    let store = Arc::new(Recorder::default());
    let kept = Arc::clone(&store);
    let look = look_for(shared_runtime(), theme);
    let mut harness = window(TALL).with_theme(theme).build_eframe(move |cc| {
        let setup = Setup {
            look,
            preferences: PreferencesSetup {
                desktop_blur,
                store: Some(kept),
                ..PreferencesSetup::default()
            },
            ..Setup::default()
        };

        let blurs = desktop_blur && Settings::default().window.blur_desktop;
        toolbox_gui_kit::theme::set_translucent(&cc.egui_ctx, blurs);

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
fn checks_turned_off_are_saved() {
    let (mut harness, store) = settings();

    harness
        .get_by_role_and_label(Role::CheckBox, "Check for updates automatically")
        .click();
    harness.run();

    let saved = &store.0.lock().saved;
    assert_eq!(
        saved.last().map(|settings| settings.status.refresh),
        Some(RefreshInterval::Off)
    );
}

fn panel_alpha(harness: &Harness<'static, App>) -> u8 {
    harness.ctx.style_of(Theme::Dark).visuals.panel_fill.a()
}

#[test]
fn the_desktop_blur_is_opt_in_and_the_switch_moves_the_translucent_background() {
    let (mut harness, store) = settings_on_with(Theme::Dark, true);
    assert_eq!(panel_alpha(&harness), 255);

    let blur = "Blur the desktop behind the window";
    harness.get_by_role_and_label(Role::CheckBox, blur).click();
    harness.run();
    assert!(store.saved().last().unwrap().window.blur_desktop);
    assert_eq!(panel_alpha(&harness), 214);

    harness.get_by_role_and_label(Role::CheckBox, blur).click();
    harness.run();
    assert!(!store.saved().last().unwrap().window.blur_desktop);
    assert_eq!(panel_alpha(&harness), 255);
}

#[test]
fn without_a_compositor_blur_the_switch_keeps_the_window_opaque() {
    let (mut harness, _store) = settings_on_with(Theme::Dark, false);
    assert_eq!(panel_alpha(&harness), 255);

    harness
        .get_by_role_and_label(Role::CheckBox, "Blur the desktop behind the window")
        .click();
    harness.run();

    assert_eq!(panel_alpha(&harness), 255);
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

#[test]
fn the_lower_cards_are_pictured() {
    let (mut harness, _store) = settings();

    harness
        .query_all_by_label("Diagnostics")
        .last()
        .unwrap()
        .scroll_to_me();
    harness.run();
    harness.snapshot("settings/lower_dark");
}
