//! What the start reads before the window: the settings file, the
//! autostart check and the log's profiler.

use std::path::PathBuf;
use std::sync::Arc;

use toolbox_cli::args::Window;
use toolbox_config::{AUTOSTART_ENTRY, Autostart, Paths, Settings};
use toolbox_core::features::boot::booted_from_snapshot;
use toolbox_gui::PreferencesSetup;
use toolbox_gui::settings::FileStore;
use toolbox_log::subscriber;
use tracing::level_filters::LevelFilter;

/// Transparent from the start, since the translucent background over a
/// blurred desktop needs an alpha surface when the window is created. Opaque
/// fills keep it opaque otherwise.
pub fn native_options(app_id: &str) -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id(app_id)
            .with_transparent(true)
            .with_inner_size([1400.0, 960.0])
            .with_min_inner_size(toolbox_gui::zoom::SMALLEST_WINDOW),
        renderer: eframe::Renderer::Wgpu,
        persist_window: true,
        ..Default::default()
    }
}

/// Asks the compositor for blur behind the window. The setting decides
/// whether the background is translucent with it, so the setting can change
/// later without asking again.
pub fn desktop_blur(cc: &eframe::CreationContext<'_>, prefs: &mut PreferencesSetup) {
    let available = toolbox_appmenu::backdrop::install(cc).translucent();
    prefs.desktop_blur = available;
    toolbox_gui_kit::theme::set_translucent(
        &cc.egui_ctx,
        available && prefs.settings.window.blur_desktop,
    );
}

/// The settings, with the log detail they ask for. A broken file still gives
/// the defaults, and the log says what was wrong.
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
    let autostart = Autostart::new(paths.autostart_dir(), PathBuf::from(AUTOSTART_ENTRY));
    if let Err(error) = autostart.follow_moves() {
        tracing::warn!("could not move the autostart link: {error}");
    }

    let enabled = autostart.is_enabled();
    let store = FileStore::with_window(paths.settings_file(), autostart, ctx.clone());

    PreferencesSetup {
        desktop_blur: false,
        settings,
        autostart: enabled,
        store: Some(Arc::new(store)),
    }
}

/// A boot into a Btrfs snapshot is for a rescue, so the login start opens
/// no window then.
pub fn skips_autostart(window: &Window) -> bool {
    let skips = window.autostart
        && std::fs::read_to_string("/proc/cmdline")
            .is_ok_and(|cmdline| booted_from_snapshot(&cmdline));

    if skips {
        tracing::info!("no start at login after a boot into a snapshot");
    }

    skips
}

/// Tracy's layer in a profiling build.
#[cfg(feature = "profiling")]
#[expect(
    clippy::unnecessary_wraps,
    reason = "a build without the feature has no layer"
)]
pub fn profiler() -> Option<subscriber::Profiler> {
    Some(toolbox_gui::profiling::layer())
}

#[cfg(not(feature = "profiling"))]
pub const fn profiler() -> Option<subscriber::Profiler> {
    None
}
