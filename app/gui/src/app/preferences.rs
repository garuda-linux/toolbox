use std::sync::Arc;

use egui::{Context, Theme};
use fluent_templates::LanguageIdentifier;
use toolbox_catalog::SchemeMode;
use toolbox_config::{Language, Settings};
use toolbox_gui_kit::{Palette, theme};
use toolbox_plan::UnitScope;

use super::App;
use crate::pages::Pages;
use crate::settings::{MotionChoice, SettingsCommand, SettingsStore, SettingsView, ThemeChoice};
use crate::{i18n, schemes};

#[derive(Debug, Default)]
pub struct PreferencesSetup {
    pub settings: Settings,
    pub autostart: bool,
    /// `None` keeps every change in memory.
    pub store: Option<Arc<dyn SettingsStore>>,
}

impl PreferencesSetup {
    fn save(&self) {
        if let Some(store) = &self.store {
            store.save(self.settings.clone());
        }
    }
}

#[derive(Debug)]
pub(super) struct Preferences {
    pub(super) setup: PreferencesSetup,
    pub(super) system_language: LanguageIdentifier,
    pub(super) desktop: (Theme, bool),
    /// The desktop's dark and light palettes, which a picked colour
    /// scheme stands in for and System brings back.
    pub(super) palettes: (Palette, Palette),
}

impl Preferences {
    pub const fn new(setup: PreferencesSetup, system_language: LanguageIdentifier) -> Self {
        Self {
            setup,
            system_language,
            desktop: (Theme::Dark, false),
            palettes: (
                Palette::fallback(Theme::Dark),
                Palette::fallback(Theme::Light),
            ),
        }
    }
}

impl App {
    pub(super) fn run_setting(&mut self, ctx: &Context, command: SettingsCommand) {
        let prefs = &mut self.prefs.setup;
        match command {
            SettingsCommand::Language(code) => {
                prefs.settings.general.language = code.and_then(|code| Language::new(code).ok());
                prefs.save();
                self.apply_language(ctx);
            }
            SettingsCommand::Theme(choice) => {
                self.state.theme = choice;
                self.apply_choices(ctx);
            }
            SettingsCommand::Scheme(mode, id) => {
                let colours = &mut prefs.settings.colours;
                match mode {
                    SchemeMode::Dark => colours.dark_scheme = id,
                    SchemeMode::Light => colours.light_scheme = id,
                }

                prefs.save();
                self.apply_palettes(ctx);
            }
            SettingsCommand::Motion(choice) => {
                self.state.motion = choice;
                self.apply_choices(ctx);
            }
            SettingsCommand::StartAtLogin(on) => {
                prefs.autostart = on;
                if let Some(store) = &prefs.store {
                    store.set_autostart(on);
                }
            }
            SettingsCommand::Refresh(interval) => {
                prefs.settings.status.refresh = interval;
                prefs.save();
                self.status.every(interval.period());
            }
            SettingsCommand::UserUnits(user) => {
                prefs.settings.services.show_user_units = user;
                prefs.save();
                self.pages.open_services_on(UnitScope::from_user(user));
            }
            SettingsCommand::CopyReports(on) => {
                prefs.settings.diagnostics.copy_to_clipboard = on;
                prefs.save();
            }
            SettingsCommand::Aur(on) => {
                prefs.settings.packages.show_aur = on;
                prefs.save();
            }
            SettingsCommand::LogLevel(level) => {
                prefs.settings.general.log_level = level;
                prefs.save();
                toolbox_log::subscriber::set_level(level.into());
            }
        }
    }

    pub(super) fn apply_language(&self, ctx: &Context) {
        let chosen = self
            .prefs
            .setup
            .settings
            .general
            .language
            .as_ref()
            .and_then(|code| code.as_str().parse::<LanguageIdentifier>().ok())
            .filter(|language| i18n::available().contains(language));

        let language = chosen.unwrap_or_else(|| self.prefs.system_language.clone());
        i18n::set_language(ctx, &language);
    }

    pub(super) fn apply_palettes(&self, ctx: &Context) {
        let colours = &self.prefs.setup.settings.colours;
        let (dark, light) =
            schemes::palettes(self.prefs.palettes, &self.catalogs.colour_schemes, colours);

        theme::apply(ctx, &dark, &light);
    }

    pub(super) fn apply_choices(&self, ctx: &Context) {
        let (desktop_theme, desktop_still) = self.prefs.desktop;
        let shown = match self.state.theme {
            ThemeChoice::FollowDesktop => desktop_theme,
            ThemeChoice::Dark => Theme::Dark,
            ThemeChoice::Light => Theme::Light,
        };

        let still = match self.state.motion {
            MotionChoice::FollowSystem => desktop_still,
            MotionChoice::Reduced => true,
            MotionChoice::Full => false,
        };

        ctx.set_theme(shown);
        theme::set_reduced_motion(ctx, still);
    }
}

pub(super) fn view(
    prefs: &Preferences,
    (theme, motion): (ThemeChoice, MotionChoice),
) -> SettingsView<'_> {
    SettingsView {
        settings: &prefs.setup.settings,
        autostart: prefs.setup.autostart,
        language: prefs
            .setup
            .settings
            .general
            .language
            .as_ref()
            .map(Language::as_str),
        theme,
        motion,
    }
}

/// The pages, with Services on the scope the settings ask for.
pub(super) fn pages_for(prefs: &Preferences) -> Pages {
    let mut pages = Pages::default();
    pages.open_services_on(UnitScope::from_user(
        prefs.setup.settings.services.show_user_units,
    ));
    pages
}
