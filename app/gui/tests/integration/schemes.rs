//! The colour scheme pickers in the app: a picked scheme takes the whole
//! palette at once, System brings the desktop's back, and both are kept.

use std::sync::Arc;

use egui::accesskit::Role;
use egui::{Key, Rect, Theme, pos2, vec2};
use egui_kittest::kittest::Queryable as _;
use egui_kittest::{Harness, SnapshotResults};
use toolbox_catalog::SchemeMode;
use toolbox_config::Settings;
use toolbox_gui::page::PageId;
use toolbox_gui::settings::SettingsStore;
use toolbox_gui::{App, PreferencesSetup, Setup};
use toolbox_gui_kit::Palette;
use toolbox_gui_kit::palette::colour;
use toolbox_gui_kit::tokens::{self, LETTER_HUES, Tokens};
use toolbox_gui_kit::widgets::{hue_of, paint_letter_tile};
use toolbox_plan::SchemeId;

use crate::common::{TALL, launch_in, window};
use crate::settings::Recorder;

const DARK: &str = "Dark colour scheme";
const LIGHT: &str = "Light colour scheme";

/// The app with the shipped catalogs in `theme`, on `page`, with
/// `settings` as the file holds them.
fn app(theme: Theme, page: PageId, settings: Settings) -> (Harness<'static, App>, Arc<Recorder>) {
    let store = Arc::new(Recorder::default());
    let kept: Arc<dyn SettingsStore> = Arc::<Recorder>::clone(&store);
    let setup = Setup {
        open: Some(page),
        catalogs: crate::discover::source_catalogs(),
        preferences: PreferencesSetup {
            settings,
            store: Some(kept),
            ..PreferencesSetup::default()
        },
        ..Setup::default()
    };

    (launch_in(window(TALL).with_theme(theme), setup), store)
}

fn picked(dark: Option<&str>, light: Option<&str>) -> Settings {
    let mut settings = Settings::default();
    settings.colours.dark_scheme = dark.map(|id| SchemeId::new(id).unwrap());
    settings.colours.light_scheme = light.map(|id| SchemeId::new(id).unwrap());
    settings
}

fn pick(harness: &mut Harness<'static, App>, picker: &str, scheme: &str) {
    harness
        .get_by_role_and_label(Role::ComboBox, picker)
        .click();
    harness.run();
    harness
        .get_by_role_and_label(Role::ListBoxOption, scheme)
        .click();
    harness.run();
}

fn desktops(theme: Theme) -> Tokens {
    Tokens::derive(&Palette::fallback(theme), theme)
}

#[test]
fn a_picked_dark_scheme_shows_at_once_with_its_own_accent() {
    let (mut harness, store) = app(Theme::Dark, PageId::Settings, Settings::default());
    assert_eq!(tokens::current(&harness.ctx), desktops(Theme::Dark));

    pick(&mut harness, DARK, "Dracula");

    let shown = tokens::current(&harness.ctx);
    assert_eq!(shown.accent, colour([0xbd, 0x93, 0xf9]));
    assert_ne!(shown.window, desktops(Theme::Dark).window);

    let saved = store.saved();
    assert_eq!(
        saved
            .last()
            .and_then(|settings| settings.colours.dark_scheme.clone()),
        Some(SchemeId::new("dracula").unwrap())
    );
}

#[test]
fn system_after_a_picked_scheme_brings_the_desktops_colours_back() {
    let (mut harness, store) = app(Theme::Dark, PageId::Settings, picked(Some("nord"), None));
    assert_eq!(
        tokens::current(&harness.ctx).accent,
        colour([0x88, 0xc0, 0xd0])
    );

    pick(&mut harness, DARK, "System");

    assert_eq!(tokens::current(&harness.ctx), desktops(Theme::Dark));
    let saved = store.saved();
    assert_eq!(
        saved
            .last()
            .map(|settings| settings.colours.dark_scheme.clone()),
        Some(None)
    );
}

#[test]
fn the_light_scheme_shows_while_the_theme_is_light() {
    let (mut harness, _store) = app(Theme::Light, PageId::Settings, Settings::default());
    crate::settings::segment(&mut harness, "Theme", "Light");

    pick(&mut harness, LIGHT, "Nord light");
    assert_eq!(
        tokens::current(&harness.ctx).accent,
        colour([0x5e, 0x81, 0xac])
    );

    // The dark pick waits for the dark theme.
    pick(&mut harness, DARK, "Sweet");
    assert_eq!(
        tokens::current(&harness.ctx).accent,
        colour([0x5e, 0x81, 0xac])
    );

    crate::settings::segment(&mut harness, "Theme", "Dark");
    assert_eq!(
        tokens::current(&harness.ctx).accent,
        colour([0xc5, 0x0e, 0xd2])
    );
}

#[test]
fn a_scheme_is_picked_with_the_keyboard_alone() {
    let (mut harness, _store) = app(Theme::Dark, PageId::Settings, Settings::default());

    harness.get_by_role_and_label(Role::ComboBox, DARK).focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    for _ in 0_u8..3 {
        harness.key_press(Key::Tab);
        harness.run();
    }

    harness.key_press(Key::Enter);
    harness.run();

    // Tab reaches System, Frappé, then Macchiato.
    assert_eq!(
        tokens::current(&harness.ctx).accent,
        colour([0xc6, 0xa0, 0xf6])
    );
}

#[test]
fn a_scheme_the_catalog_lost_keeps_the_desktops_colours() {
    let (harness, _store) = app(
        Theme::Dark,
        PageId::Settings,
        picked(Some("removed-scheme"), None),
    );

    assert_eq!(tokens::current(&harness.ctx), desktops(Theme::Dark));
}

#[test]
fn pages_under_picked_schemes() {
    let mut results = SnapshotResults::new();

    let (mut dark, _store) = app(Theme::Dark, PageId::Settings, picked(Some("nord"), None));
    results.add(dark.try_snapshot("settings/scheme_dark"));

    let (mut light, _light_store) = app(
        Theme::Light,
        PageId::Settings,
        picked(None, Some("catppuccin-latte")),
    );

    crate::settings::segment(&mut light, "Theme", "Light");
    rest(&mut light);
    results.add(light.try_snapshot("settings/scheme_light"));

    let (mut home, _home_store) = app(Theme::Dark, PageId::Home, picked(Some("tokyo-night"), None));
    results.add(home.try_snapshot("shell/home_scheme_dark"));

    let (mut day, _day_store) = app(
        Theme::Light,
        PageId::Home,
        picked(None, Some("tokyo-night-day")),
    );

    rest(&mut day);
    results.add(day.try_snapshot("shell/home_scheme_light"));
}

#[test]
fn the_german_pickers_fit() {
    let store = Arc::new(Recorder::default());
    let setup = Setup {
        open: Some(PageId::Settings),
        system_language: "de-DE".parse().unwrap(),
        catalogs: crate::discover::source_catalogs(),
        preferences: PreferencesSetup {
            settings: picked(Some("catppuccin-macchiato"), Some("dracula-alucard")),
            store: Some(store),
            ..PreferencesSetup::default()
        },
        ..Setup::default()
    };

    let mut harness = launch_in(window(TALL), setup);
    harness.snapshot("settings/schemes_german");
}

/// The pointer off the window, so no hover shows in a picture.
fn rest(harness: &mut Harness<'static, App>) {
    harness.event(egui::Event::PointerGone);
    harness.run();
}

/// The eight letter tiles on a card under the fallback, Nord and Dracula,
/// top to bottom, so the schemes' softness shows side by side.
#[test]
fn letter_tiles_take_each_scheme_s_softness() {
    const TILE: f32 = 40.0;
    const GAP: f32 = 8.0;

    let catalog = crate::discover::source_catalogs().colour_schemes;
    let scheme = |id: &str| {
        let id = SchemeId::new(id).unwrap();
        toolbox_gui::schemes::palette_of(catalog.find(&id, SchemeMode::Dark).unwrap())
    };

    let rows = [
        Palette::fallback(Theme::Dark),
        scheme("nord"),
        scheme("dracula"),
    ];

    // One package name per hue, so each row shows all eight.
    let mut names: [Option<String>; LETTER_HUES] = Default::default();
    for number in 0_u32.. {
        let name = format!("app-{number}");
        if let Some(slot @ None) = names.get_mut(hue_of(&name)) {
            *slot = Some(name);
        }

        if names.iter().all(Option::is_some) {
            break;
        }
    }

    let names = names.map(Option::unwrap_or_default);
    let letters = ["A", "B", "C", "D", "E", "F", "G", "H"];
    let step = TILE + GAP;
    let width = step.mul_add(f32::from(u8::try_from(LETTER_HUES).unwrap()), GAP);

    let mut harness = toolbox_gui_kit::testing::harness(width, step.mul_add(3.0, GAP))
        .with_theme(Theme::Dark)
        .build_ui(move |ui| {
            let light = Palette::fallback(Theme::Light);
            for (row, palette) in (0_u8..).zip(&rows) {
                toolbox_gui_kit::theme::apply(ui.ctx(), palette, &light);
                let top = step * f32::from(row);
                let band = Rect::from_min_size(pos2(0.0, top), vec2(width, step + GAP));
                ui.painter()
                    .rect_filled(band, 0.0, tokens::current(ui.ctx()).card);

                for (column, (name, letter)) in (0_u8..).zip(names.iter().zip(letters)) {
                    let left = step.mul_add(f32::from(column), GAP);
                    let at = Rect::from_min_size(pos2(left, top + GAP), vec2(TILE, TILE));
                    paint_letter_tile(ui, at, 6, letter, name);
                }
            }
        });

    harness.run();
    harness.snapshot("settings/letter_tiles_schemes");
}
