//! The settings file, read once at start.

use std::path::PathBuf;
use std::sync::Arc;

use toolbox_config::{Autostart, DESKTOP_FILE, Paths, Settings};
use toolbox_gui::PreferencesSetup;
use toolbox_gui::settings::FileStore;
use toolbox_log::subscriber;
use tracing::level_filters::LevelFilter;

pub fn load_settings(paths: &Paths) -> Settings {
    let loaded = Settings::load(&paths.settings_file());

    subscriber::set_level(LevelFilter::from(loaded.settings.general.log_level));

    if let Some(problem) = loaded.problem {
        tracing::warn!("{problem}");
    }

    loaded.settings
}

/// The Settings page's start: the file's settings, the autostart link,
/// and both written back on a thread of their own.
pub fn preferences(paths: &Paths, settings: Settings, ctx: &egui::Context) -> PreferencesSetup {
    let autostart = Autostart::new(paths.autostart_dir(), PathBuf::from(DESKTOP_FILE));
    let enabled = autostart.is_enabled();
    let store = FileStore::with_window(paths.settings_file(), autostart, ctx.clone());

    PreferencesSetup {
        settings,
        autostart: enabled,
        store: Some(Arc::new(store)),
    }
}
